// src/state.rs

use crate::data::market_data::MarketDataManager;

pub struct AppState {
    pub market_manager: MarketDataManager,
}

impl AppState {
    pub fn new(market_manager: MarketDataManager) -> Self {
        Self {
            market_manager,
        }
    }
}