// trading-core/src/backtest/mod.rs

pub mod sma;
pub mod types;
pub mod engine;
pub mod metrics;

use std::collections::HashMap;
pub use types::*;

use crate::data::market_data::MarketDataPoint;
pub trait Strategy: Send {
    fn on_data(&mut self, data: &MarketDataPoint, portfolio: &Portfolio) -> Vec<Order>;
    fn get_parameters(&self) -> &HashMap<String, String>;
    fn get_type(&self) -> StrategyType;
}