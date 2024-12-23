// src-tauri/src/commands/market.rs
use tauri::State;
use trading_core::data::market_data::MarketDataPoint;
use chrono::{DateTime, Utc};
use crate::state::AppState;

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
    limit: u32,
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
