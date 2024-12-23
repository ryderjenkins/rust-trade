// src-tauri/src/commands/market.rs
use tauri::State;
use trading_core::data::market_data::MarketDataPoint;
use chrono::{DateTime,Duration, Utc};
use crate::state::AppState;
use rust_decimal::Decimal;
use std::str::FromStr;
use trading_core::{
    backtest::{
        strategy::sma_cross::SimpleMovingAverageCrossStrategy,
        types::BacktestConfig,
    },
    data::market_data::MarketDataManager,
};

use trading_core::backtest::engine::engine::BacktestEngine;

#[derive(serde::Serialize)]
pub struct BacktestResponse {
    total_return: Decimal,
    total_trades: u32,
    winning_trades: u32,
    losing_trades: u32,
    max_drawdown: Decimal,
    trades: Vec<TradeResponse>,
    equity_curve: Vec<EquityPoint>, 
}

#[derive(serde::Serialize)]
pub struct TradeResponse {
    timestamp: String,
    side: String,
    symbol: String,
    quantity: Decimal,
    price: Decimal,
    commission: Decimal,
}

#[derive(serde::Serialize)]
pub struct EquityPoint {
    timestamp: String,
    value: Decimal,
}

#[derive(serde::Serialize)]
pub struct MarketOverview {
    pub price: f64,
    pub price_change_24h: f64,
    pub volume_24h: f64,
}

#[tauri::command]
pub async fn get_market_data(
    state: State<'_, AppState>,
    symbol: String,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> Result<Vec<MarketDataPoint>, String> {
    state.market_manager
        .get_market_data(&symbol, start_time, end_time)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_latest_price(
    state: State<'_, AppState>,
    symbol: String,
) -> Result<f64, String> {
    state.market_manager
        .get_latest_price(&symbol)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_candlestick_data(
    state: tauri::State<'_, AppState>,
    symbol: String,
    interval: String,
    start_time: Option<String>,
    end_time: Option<String>,   
) -> Result<Vec<MarketDataPoint>, String> {
    let start_time = start_time
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").ok());
    let end_time = end_time
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").ok());

    state.market_manager
        .get_candlestick_data(&symbol, &interval, start_time, end_time)
        .await
        .map_err(|e| e.to_string())
}


#[tauri::command]
pub async fn get_market_overview(
    state: tauri::State<'_, AppState>,
    symbol: String,
) -> Result<MarketOverview, String> {
    // 获取最新价格
    let latest_price = state.market_manager
        .get_latest_price(&symbol)
        .await
        .map_err(|e| e.to_string())?;
    
    // 获取24小时的数据来计算价格变化
    let end_time = chrono::Utc::now();
    let start_time = end_time - chrono::Duration::hours(24);
    
    let market_data = state.market_manager
        .get_market_data(&symbol, start_time, end_time)
        .await
        .map_err(|e| e.to_string())?;

    let price_24h_ago = market_data
        .first()
        .map(|data| data.price)
        .unwrap_or(latest_price);

    let price_change = ((latest_price - price_24h_ago) / price_24h_ago * 100.0)
        .round()
        .abs();

    let volume_24h: f64 = market_data
        .iter()
        .map(|data| data.volume)
        .sum();

    Ok(MarketOverview {
        price: latest_price,
        price_change_24h: price_change,
        volume_24h,
    })
}

#[tauri::command]
pub async fn run_backtest(
    state: tauri::State<'_, AppState>,
    symbol: String,
    days: i64,
    initial_capital: String,
    commission_rate: String,
    short_period: usize,
    long_period: usize,
) -> Result<BacktestResponse, String> {

    let market_data_manager = &state.market_manager;
    let pool = market_data_manager.get_pool();
    let market_data_manager = MarketDataManager::new(pool);
    
    let end_time = Utc::now();
    let start_time = end_time - Duration::days(days);
    
    let config = BacktestConfig {
        symbol: symbol.clone(),
        start_time,
        end_time,
        initial_capital: Decimal::from_str(&initial_capital).map_err(|e| e.to_string())?,
        commission_rate: Decimal::from_str(&commission_rate).map_err(|e| e.to_string())?,
    };

    let position_size = Decimal::from_str(&initial_capital).unwrap() / Decimal::from(10);
    let mut strategy = SimpleMovingAverageCrossStrategy::new(
        symbol.clone(),
        short_period,
        long_period,
        position_size,
    );

    let mut engine = BacktestEngine::new(market_data_manager, config);
    let result = engine.run(&mut strategy).await.map_err(|e| e.to_string())?;

    let trades = result.trades.into_iter().map(|trade| TradeResponse {
        timestamp: trade.timestamp.to_rfc3339(),
        side: match trade.side {
            trading_core::backtest::types::OrderSide::Buy => "Buy".to_string(),
            trading_core::backtest::types::OrderSide::Sell => "Sell".to_string(),
        },
        symbol: trade.symbol,
        quantity: trade.quantity,
        price: trade.price,
        commission: trade.commission,
    }).collect();

    Ok(BacktestResponse {
        total_return: result.total_return,
        total_trades: result.total_trades,
        winning_trades: result.winning_trades,
        losing_trades: result.losing_trades,
        max_drawdown: result.max_drawdown,
        trades,
        equity_curve: engine.get_equity_curve().into_iter().map(|(timestamp, value)| EquityPoint {
            timestamp: timestamp.to_rfc3339(),
            value,
        }).collect(),
    })
}