use std::sync::Arc;
use trading_core::{
    data::{
        database::Database,
        market_data::MarketDataManager,
    },
    config::Settings,
};

pub struct AppState {
    pub market_manager: Arc<MarketDataManager>,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let settings = Settings::new()?;
        let database = Database::new(&settings.database).await?;
        
        Ok(Self {
            market_manager: Arc::new(MarketDataManager::new(database.pool)),
        })
    }
}