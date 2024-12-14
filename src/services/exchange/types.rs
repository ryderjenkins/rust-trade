use crate::data::market_data::MarketDataPoint;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExchangeError {
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Invalid symbol: {0}")]
    InvalidSymbol(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Authentication error: {0}")]
    AuthError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookLevel {
    pub price: Decimal,
    pub quantity: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub last_price: Decimal,
    pub bid_price: Decimal,
    pub ask_price: Decimal,
    pub volume_24h: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub price: Decimal,
    pub quantity: Decimal,
    pub is_buyer_maker: bool,
}

#[async_trait::async_trait]
pub trait Exchange: Send + Sync {
    /// 获取交易对的最新行情
    async fn get_ticker(&self, symbol: &str) -> Result<Ticker, ExchangeError>;
    
    /// 获取交易对的订单簿
    async fn get_orderbook(&self, symbol: &str, limit: u32) -> Result<OrderBook, ExchangeError>;
    
    /// 获取最近的成交记录
    async fn get_recent_trades(&self, symbol: &str, limit: u32) -> Result<Vec<Trade>, ExchangeError>;
    
    /// 获取K线数据
    async fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<u32>,
    ) -> Result<Vec<MarketDataPoint>, ExchangeError>;
    
    /// 订阅实时市场数据
    async fn subscribe_market_data(
        &self,
        symbols: &[String],
        callback: Box<dyn Fn(MarketDataPoint) + Send + Sync>,
    ) -> Result<(), ExchangeError>;
}