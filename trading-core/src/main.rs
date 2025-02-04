use bigdecimal::{FromPrimitive, Zero};
use clap::{Parser, Subcommand};
use dotenv::dotenv;
use tracing::{info, error};
use std::sync::Arc;
use std::str::FromStr;
use chrono::{Duration, Utc};
use rust_decimal::Decimal;

use trading_core::{
   backtest::{engine::BacktestEngine, sma::SMAStrategy, types::OrderSide, BacktestConfig}, 
   config::Settings, data::{database::Database, market_data::MarketDataManager}, 
   exchange::binance::BinanceSpot, market_data_collector::MarketDataCollector
};

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
   // 加载环境变量和初始化日志
   dotenv().ok();
   tracing_subscriber::fmt()
       .with_max_level(tracing::Level::DEBUG)
       .init();

   // 加载配置并初始化数据库
   let settings = Settings::new()?;
   let database = Database::new(&settings.database).await?;
   database.check_connection().await?;
   info!("Database connection established");

   match Cli::parse().command.unwrap_or(Commands::Server) {
       Commands::Server => {
           // 初始化交易所和数据收集器
           let exchange = BinanceSpot::new(None, None);
           let collector = Arc::new(MarketDataCollector::new(
               Box::new(exchange),
               MarketDataManager::new(database.pool.clone()),
               vec!["BTCUSDT".to_string()],
           ));

           // 启动收集器
           let collector_clone = collector.clone();
           let handle = tokio::spawn(async move {
               if let Err(e) = collector_clone.start().await {
                   error!("Market data collector error: {}", e);
               }
           });

           info!("Market data collector started. Press Ctrl+C to stop.");

           // 等待中断信号
           tokio::signal::ctrl_c().await?;
           info!("Shutting down server...");
           collector.stop();
           handle.await?;
           info!("Server shutdown complete");
       }

       Commands::Backtest { 
           symbol, 
           days, 
           initial_capital, 
           commission_rate,
           short_period,
           long_period,
       } => {
           let market_data = MarketDataManager::new(database.pool);
           
           // 设置回测时间范围
           let start_time = Utc::now() - Duration::days(days);
           let end_time = Utc::now();
           
           // 检查数据可用性
           let data = market_data.get_market_data(&symbol, start_time, end_time).await?;
           if data.is_empty() {
               error!("No historical data found for {} in the specified time range", symbol);
               return Err("Insufficient historical data for backtest".into());
           }
           info!("Found {} historical data points", data.len());

           // 创建回测配置
           let config = BacktestConfig {
               start_time,
               end_time,
               initial_capital: Decimal::from_str(&initial_capital)?,
               symbol: symbol.clone(),
               commission_rate: Decimal::from_str(&commission_rate)?,
           };

           // 创建策略实例
           let position_size = match data.first() {
            Some(first_data) => {
                // 使用初始资金的 10% 除以当前价格，得到数量
                let capital = Decimal::from_str(&initial_capital)?;
                (capital * Decimal::from_f64(0.1).unwrap()) / Decimal::from_f64(first_data.price).unwrap()
            }
            None => Decimal::zero()
        };
           let strategy = SMAStrategy::new(
               symbol.clone(),
               short_period,
               long_period,
               position_size,
           );

           // 运行回测
           let mut engine = BacktestEngine::new(market_data, config);
           let result = engine.run_strategy(Box::new(strategy)).await?;

           // 打印回测结果
           println!("\nBacktest Results:");
           println!("Total Return: {}%", result.metrics.total_return);
           println!("Total Trades: {}", result.metrics.total_trades);
           println!("Win Rate: {}%", result.metrics.win_rate);
           println!("Sharpe Ratio: {}", result.metrics.sharpe_ratio);
           println!("Max Drawdown: {}%", result.metrics.max_drawdown);
           println!("\nTrade History:");
           for trade in result.trades {
               println!(
                   "{} {} {} @ {}",
                   trade.timestamp.format("%Y-%m-%d %H:%M:%S"),
                   if trade.side == OrderSide::Buy { "BUY" } else { "SELL" },
                   trade.quantity,
                   trade.price
               );
           }
       }
   }

   Ok(())
}