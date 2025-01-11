// src-tauri/src/commands/market.rs

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

#[tauri::command]
pub async fn run_backtest<'a>(
    state: State<'a, AppState>,
    request: BacktestRequest,
) -> Result<BacktestResponse, String> {
    let market_data = MarketDataManager::new(state.market_manager.get_pool());
    
    // 从参数中获取策略参数
    let short_period = request.parameters.get("short_period")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(5);
        
    let long_period = request.parameters.get("long_period")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(20);
        
    let position_size_percent = request.parameters.get("position_size_percent")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(10.0);

    // 计算仓位大小
    let position_size = request.config.initial_capital
        * Decimal::from_f64(position_size_percent / 100.0).unwrap();
    
    // 根据策略类型创建策略
    let strategy = match request.strategy_type {
        StrategyType::SMACross => {
            SMAStrategy::new(
                request.config.symbol.clone(),
                short_period,
                long_period,
                position_size,
            )
        },
        _ => return Err("Unsupported strategy type".to_string()),
    };

    // 运行回测
    let mut engine = BacktestEngine::new(market_data, request.config);
    let result = engine.run_strategy(Box::new(strategy))
        .await
        .map_err(|e| e.to_string())?;

    // 转换结果为响应格式
    Ok(BacktestResponse {
        total_return: result.metrics.total_return.to_string(),
        sharpe_ratio: result.metrics.sharpe_ratio,
        max_drawdown: result.metrics.max_drawdown.to_string(),
        win_rate: result.metrics.win_rate.to_string(),
        total_trades: result.metrics.total_trades,
        equity_curve: result.equity_curve,
        trades: result.trades.into_iter().map(|trade| TradeResponse {
            timestamp: trade.timestamp.to_rfc3339(),
            symbol: trade.symbol,
            side: match trade.side {
                trading_core::backtest::types::OrderSide::Buy => "Buy".to_string(),
                trading_core::backtest::types::OrderSide::Sell => "Sell".to_string(),
            },
            quantity: trade.quantity.to_string(),
            price: trade.price.to_string(),
            commission: trade.commission.to_string(),
        }).collect(),
    })
}