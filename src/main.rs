// src/main.rs
mod api;
mod backtest;
mod config;
mod data;
mod services;
mod ai;

use ai::llm::risk::RiskManager;
use ai::llm::strategy_generator::LLMStrategy;
use backtest::OrderSide;
use clap::{Parser, Subcommand};
use dotenv::dotenv;
use services::market_data_collector::MarketDataCollector;
use tracing::{info, error, Level};
use std::net::SocketAddr;
use tokio::signal;
use crate::api::ApiServer;
use crate::data::database::Database;
use crate::services::exchange::binance::BinanceSpot;
use crate::config::Settings;
use crate::backtest::{
    engine::engine::BacktestEngine,
    strategy::sma_cross::SimpleMovingAverageCrossStrategy,
    types::BacktestConfig,
};
use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "rust-trade")]
#[command(about = "A Rust trading system with backtest capabilities")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the trading server
    Server,
    /// Run backtest with specified parameters
    Backtest {
        #[arg(short, long, default_value = "BTCUSDT")]
        symbol: String,
        #[arg(short, long, default_value = "30")]
        days: i64,
        #[arg(short, long, default_value = "10000.0")]
        initial_capital: String,
        #[arg(short, long, default_value = "0.001")]
        commission_rate: String,
        #[arg(long, default_value = "5")]
        short_period: usize,
        #[arg(long, default_value = "20")]
        long_period: usize,
    },
    /// Run backtest with LLM strategy
    LLMBacktest {
        #[arg(short, long, default_value = "BTCUSDT")]
        symbol: String,
        #[arg(short, long, default_value = "1")]
        days: i64,
        #[arg(short, long, default_value = "10000.0")]
        initial_capital: String,
        #[arg(short, long, default_value = "0.001")]
        commission_rate: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载 .env 文件
    dotenv().ok();

    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // 解析命令行参数
    let cli = Cli::parse();

    // 加载配置
    let settings = Settings::new()?;
    info!("Configuration loaded successfully");

    // 初始化数据库连接
    let database = Database::new(&settings.database).await?;
    database.check_connection().await?;
    info!("Database connection established");

    match cli.command.unwrap_or(Commands::Server) {
        Commands::Server => {
            run_server(database, settings).await?;
        }
        Commands::Backtest { 
            symbol, 
            days, 
            initial_capital, 
            commission_rate,
            short_period,
            long_period,
        } => {
            run_backtest(
                database.pool,
                symbol,
                days,
                initial_capital,
                commission_rate,
                short_period,
                long_period,
            ).await?;
        }
        Commands::LLMBacktest { 
            symbol, 
            days, 
            initial_capital, 
            commission_rate,
        } => {
            run_llm_backtest(
                database.pool,
                symbol,
                days,
                initial_capital,
                commission_rate,
            ).await?;
        }
    }

    Ok(())
}

async fn run_server(database: Database, settings: Settings) -> Result<(), Box<dyn std::error::Error>> {
    // 初始化交易所服务
    let exchange = BinanceSpot::new(None, None);
    info!("Exchange service initialized");

    // 创建并启动市场数据收集器
    let collector = MarketDataCollector::new(
        Box::new(exchange.clone()),
        data::market_data::MarketDataManager::new(database.pool.clone()),
        vec!["BTCUSDT".to_string()],
    );

    let collector = Arc::new(collector);
    let collector_clone = collector.clone();
    
    let collector_handle = tokio::spawn(async move {
        if let Err(e) = collector_clone.start().await {
            error!("Market data collector error: {}", e);
        }
    });
    info!("Market data collector started");

    // 启动 API 服务器
    let addr = SocketAddr::from(([127, 0, 0, 1], settings.api.port));
    let api_server = ApiServer::new(Box::new(exchange), addr);
    
    info!("Starting API server on {}", addr);

    // 关闭处理
    let shutdown_signal = async {
        signal::ctrl_c()
            .await
            .expect("Failed to initialize shutdown signal handler");
        info!("Shutdown signal received");
        collector.stop();
    };

    tokio::select! {
        result = api_server.run() => {
            if let Err(e) = result {
                error!("API server error: {}", e);
            }
        }
        _ = shutdown_signal => {
            info!("Shutting down services");
            collector_handle.abort();
        }
    }

    info!("Server shutdown complete");
    Ok(())
}

async fn run_backtest(
    pool: sqlx::PgPool,
    symbol: String,
    days: i64,
    initial_capital: String,
    commission_rate: String,
    short_period: usize,
    long_period: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let market_data = data::market_data::MarketDataManager::new(pool);
    
    // 检查是否有足够的历史数据
    let start_time = Utc::now() - Duration::days(days);
    let end_time = Utc::now();
    
    info!("Checking historical data availability for {}", symbol);
    let data = market_data.get_market_data(&symbol, start_time, end_time).await?;
    
    if data.is_empty() {
        error!("No historical data found for {} in the specified time range", symbol);
        info!("Please ensure market data collector is running and has collected enough data");
        return Err("Insufficient historical data for backtest".into());
    }

    info!("Found {} historical data points", data.len());

    // 设置回测配置
    let config = BacktestConfig {
        start_time,
        end_time,
        initial_capital: Decimal::from_str(&initial_capital)?,
        symbol: symbol.clone(),
        commission_rate: Decimal::from_str(&commission_rate)?,
    };

    // 创建策略实例
    let position_size = Decimal::from_str(&initial_capital)? / Decimal::from(10);
    let mut strategy = SimpleMovingAverageCrossStrategy::new(
        symbol,
        short_period,
        long_period,
        position_size,
    );

    // 创建并运行回测引擎
    let mut engine = BacktestEngine::new(market_data, config);
    let result = engine.run(&mut strategy).await?;

    // 打印回测结果
    println!("\nBacktest results:");
    println!("Total return: {}%", result.total_return);
    println!("Total number of trades: {}", result.total_trades);
    println!("Profitable trades: {}", result.winning_trades);
    println!("Losing trades: {}", result.losing_trades);
    println!("Maximum drawdown: {}%", result.max_drawdown);

    println!("\nDetailed trading records:");
    for (i, trade) in result.trades.iter().enumerate() {
        println!(
            "{}. {} {} {} @ {} (handling fee: {})",
            i + 1,
            trade.timestamp.format("%Y-%m-%d %H:%M:%S"),
            match trade.side {
                backtest::types::OrderSide::Buy => "Buy",
                backtest::types::OrderSide::Sell => "Sell",
            },
            trade.quantity,
            trade.price,
            trade.commission
        );
    }

    Ok(())
}

async fn run_llm_backtest(
    pool: sqlx::PgPool,
    symbol: String,
    days: i64,
    initial_capital: String,
    commission_rate: String,
 ) -> Result<(), Box<dyn std::error::Error>> {
    let market_data = data::market_data::MarketDataManager::new(pool);
    
    // 检查是否有足够的历史数据
    let start_time = Utc::now() - Duration::days(days);
    let end_time = Utc::now();
    
    info!("Checking historical data availability for {}", symbol);
    let data = market_data.get_market_data(&symbol, start_time, end_time).await?;
    
    if data.is_empty() {
        error!("No historical data found for {} in the specified time range", symbol);
        info!("Please ensure market data collector is running and has collected enough data");
        return Err("Insufficient historical data for backtest".into());
    }
 
    info!("Found {} historical data points", data.len());
 
    // 获取 OpenAI API key
    let api_key = std::env::var("OPENAI_KEY")
        .expect("OPENAI_KEY must be set in environment");
 
    // 创建风险管理器
    let risk_manager = RiskManager::new(
        Decimal::from_str("0.1")?,     // 最大回撤 10%
        Decimal::from_str("0.02")?,    // 每笔交易风险 2%
        Decimal::from_str("0.1")?,
    );
 
    // 设置回测配置
    let config = BacktestConfig {
        start_time,
        end_time,
        initial_capital: Decimal::from_str(&initial_capital)?,
        symbol: symbol.clone(),
        commission_rate: Decimal::from_str(&commission_rate)?,
    };
 
    let position_size = Decimal::from_str(&initial_capital)? / Decimal::from(10);
 
    // 创建 LLM 策略实例
    let mut strategy = LLMStrategy::new(
        symbol.clone(),
        api_key,
        risk_manager,
        position_size,
    );
 
    // 创建并运行回测引擎
    let mut engine = BacktestEngine::new(market_data, config);
    info!("Starting LLM strategy backtest...");
    let result = engine.run(&mut strategy).await?;
 
    // 打印回测结果
    println!("\nLLM Strategy Backtest Results:");
    println!("--------------------------------");
    println!("Symbol: {}", symbol);
    println!("Period: {} days", days);
    println!("Initial Capital: {}", initial_capital);
    println!("--------------------------------");
    println!("Total Return: {}%", result.total_return);
    println!("Total Trades: {}", result.total_trades);
    println!("Winning Trades: {}", result.winning_trades);
    println!("Losing Trades: {}", result.losing_trades);
    println!("Maximum Drawdown: {}%", result.max_drawdown);
    println!("Win Rate: {:.2}%", 
        if result.total_trades > 0 {
            (result.winning_trades as f64 / result.total_trades as f64) * 100.0
        } else {
            0.0
        }
    );
 
    println!("\nDetailed Trade History:");
    println!("--------------------------------");
    for (i, trade) in result.trades.iter().enumerate() {
        println!(
            "{}. {} | {} | Quantity: {} | Price: {} | Fee: {}",
            i + 1,
            trade.timestamp.format("%Y-%m-%d %H:%M:%S"),
            match trade.side {
                OrderSide::Buy => "BUY ",
                OrderSide::Sell => "SELL",
            },
            trade.quantity,
            trade.price,
            trade.commission
        );
    }
 
    Ok(())
 }