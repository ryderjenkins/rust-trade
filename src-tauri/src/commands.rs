// src-tauri/src/commands.rs

use crate::state::AppState;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use tauri::State;
use trading_core::{
    backtest::{
        engine::BacktestEngine,
        sma::SMAStrategy,
        types::{BacktestRequest, BacktestResponse, TradeResponse, StrategyType}
    },
    data::market_data::MarketDataManager,
};
use tracing::{info, error, debug};

#[tauri::command]
pub async fn run_backtest<'a>(
    state: State<'a, AppState>,
    request: BacktestRequest,
) -> Result<BacktestResponse, String> {
    let market_data = MarketDataManager::new(state.market_manager.get_pool());

    // 首先获取当前价格数据
    let data = market_data
        .get_market_data(
            &request.config.symbol,
            request.config.start_time,
            request.config.end_time,
        )
        .await
        .map_err(|e| e.to_string())?;

    if data.is_empty() {
        return Err("No historical data available".to_string());
    }

    // 计算实际的交易数量而不是金额
    let first_price = Decimal::from_f64(data[0].price)
        .ok_or("Failed to convert price")?;

    let position_size_percent = request.parameters
        .get("position_size_percent")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(10.0);

    // 计算实际的交易数量
    let position_size = (request.config.initial_capital * 
        Decimal::from_f64(position_size_percent / 100.0).unwrap()) / 
        first_price;

    info!(
        "Position calculation: capital={}, percent={}, price={}, quantity={}", 
        request.config.initial_capital,
        position_size_percent,
        first_price,
        position_size
    );

    let strategy = match request.strategy_type {
        StrategyType::SMACross => {
            SMAStrategy::new(
                request.config.symbol.clone(),
                request.parameters.get("short_period")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(5),
                request.parameters.get("long_period")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(20),
                position_size, // 这里传入的是数量而不是金额
            )
        },
        _ => return Err("Unsupported strategy type".to_string()),
    };

    // 运行回测
    info!("Initializing backtest engine");
    let mut engine = BacktestEngine::new(market_data, request.config.clone());
    
    info!("Starting backtest");
    let result = match engine.run_strategy(Box::new(strategy)).await {
        Ok(res) => {
            info!("Backtest completed successfully");
            debug!("Backtest metrics: {:?}", res.metrics);
            res
        },
        Err(e) => {
            error!("Backtest failed: {}", e);
            return Err(e.to_string());
        }
    };

    // 转换结果为响应格式
    info!("Converting results to response format");
    let response = BacktestResponse {
        total_return: result.metrics.total_return.to_string(),
        sharpe_ratio: result.metrics.sharpe_ratio,
        max_drawdown: result.metrics.max_drawdown.to_string(),
        win_rate: result.metrics.win_rate.to_string(),
        total_trades: result.metrics.total_trades,
        equity_curve: result.equity_curve,
        trades: result.trades.into_iter().map(|trade| {
            debug!("Processing trade: {:?}", trade);
            TradeResponse {
                timestamp: trade.timestamp.to_rfc3339(),
                symbol: trade.symbol,
                side: match trade.side {
                    trading_core::backtest::types::OrderSide::Buy => "Buy".to_string(),
                    trading_core::backtest::types::OrderSide::Sell => "Sell".to_string(),
                },
                quantity: trade.quantity.to_string(),
                price: trade.price.to_string(),
                commission: trade.commission.to_string(),
            }
        }).collect(),
    };

    info!("Backtest response prepared successfully");
    Ok(response)
}