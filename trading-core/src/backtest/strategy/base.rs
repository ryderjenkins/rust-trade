use crate::data::market_data::MarketDataPoint;
use crate::backtest::types::{Order, Portfolio};

pub trait Strategy: Send {
    fn on_data(&mut self, data: &MarketDataPoint, portfolio: &Portfolio) -> Vec<Order>;
}