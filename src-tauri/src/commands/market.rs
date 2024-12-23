// src-tauri/src/commands/market.rs
use tauri::State;
use trading_core::data::market_data::MarketDataPoint;
use chrono::{DateTime, Utc};
use crate::state::AppState;

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