use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub initial_capital: Decimal,
    pub symbol: String,
    pub commission_rate: Decimal,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub quantity: Decimal,
    pub average_entry_price: Decimal,
}

#[derive(Debug, Clone)]
pub struct Portfolio {
    pub cash: Decimal,
    pub positions: HashMap<String, Position>,
    pub total_value: Decimal,
}

#[derive(Debug, Clone)]
pub enum OrderType {
    Market,
}

#[derive(Debug, Clone)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub symbol: String,
    pub order_type: OrderType,
    pub side: OrderSide,
    pub quantity: Decimal,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Trade {
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: Decimal,
    pub price: Decimal,
    pub timestamp: DateTime<Utc>,
    pub commission: Decimal,
}

#[derive(Debug, Clone)]
pub struct BacktestResult {
    pub total_return: Decimal,
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub max_drawdown: Decimal,
    pub trades: Vec<Trade>,
}