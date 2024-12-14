use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MarketTickerResponse {
    pub symbol: String,
    pub price: Decimal,
    pub timestamp: DateTime<Utc>,
    pub volume_24h: Decimal,
    pub high_24h: Decimal,
    pub low_24h: Decimal,
}

#[derive(Debug, Serialize)]
pub struct OrderBookResponse {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub bids: Vec<(Decimal, Decimal)>,  
    pub asks: Vec<(Decimal, Decimal)>,
}

#[derive(Debug, Deserialize)]
pub struct KlineQuery {
    pub symbol: String,
    pub interval: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
}