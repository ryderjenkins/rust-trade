use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Serialize)]
pub struct MarketOverview {
    pub price: f64,
    pub volume_24h: f64,
    pub price_change_24h: f64,
    pub high_24h: f64,
    pub low_24h: f64,
}

#[derive(Serialize)]
pub struct TradingStats {
    pub total_trades: i32,
    pub win_rate: f64,
    pub profit_loss: f64,
    pub avg_profit_per_trade: f64,
}

#[derive(Serialize)]
pub struct PortfolioStats {
    pub total_value: f64,
    pub daily_pnl: f64,
    pub asset_allocation: Vec<(String, f64)>,
    pub performance_history: Vec<(DateTime<Utc>, f64)>,
}

#[tauri::command]
pub async fn get_market_overview(
    state: tauri::State<'_, AppState>,
    symbol: String,
) -> Result<MarketOverview, String> {
    let pool = &state.market_manager.pool;
    
    // 查询24小时数据
    let result = sqlx::query!(
        r#"
        SELECT 
            (SELECT price FROM market_data 
             WHERE symbol = $1 
             ORDER BY timestamp DESC 
             LIMIT 1) as current_price,
            SUM(volume) as total_volume,
            MAX(high) as high_24h,
            MIN(low) as low_24h,
            (SELECT price FROM market_data 
             WHERE symbol = $1 
             AND timestamp <= NOW() - INTERVAL '24 hours'
             ORDER BY timestamp DESC 
             LIMIT 1) as price_24h_ago
        FROM market_data
        WHERE symbol = $1
        AND timestamp >= NOW() - INTERVAL '24 hours'
        "#,
        symbol
    )
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    let price_change = ((result.current_price.unwrap_or_default() - result.price_24h_ago.unwrap_or_default())
        / result.price_24h_ago.unwrap_or(1.0)) * 100.0;

    Ok(MarketOverview {
        price: result.current_price.unwrap_or_default(),
        volume_24h: result.total_volume.unwrap_or_default(),
        price_change_24h: price_change,
        high_24h: result.high_24h.unwrap_or_default(),
        low_24h: result.low_24h.unwrap_or_default(),
    })
}

#[tauri::command]
pub async fn get_trading_stats(
    state: tauri::State<'_, AppState>
) -> Result<TradingStats, String> {
    // 这里添加交易统计查询逻辑
    Ok(TradingStats {
        total_trades: 150,
        win_rate: 65.0,
        profit_loss: 25000.0,
        avg_profit_per_trade: 166.67,
    })
}

#[tauri::command]
pub async fn get_portfolio_stats(
    state: tauri::State<'_, AppState>
) -> Result<PortfolioStats, String> {
    // 这里添加投资组合统计查询逻辑
    Ok(PortfolioStats {
        total_value: 100000.0,
        daily_pnl: 2.5,
        asset_allocation: vec![
            ("BTC".to_string(), 45.0),
            ("ETH".to_string(), 35.0),
            ("USDT".to_string(), 20.0),
        ],
        performance_history: vec![
            // 添加历史表现数据
        ],
    })
} 