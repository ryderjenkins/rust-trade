use super::types::{Exchange, ExchangeError};
use crate::data::market_data::{MarketDataManager, MarketDataPoint};
use tokio::sync::broadcast;
use tracing::{debug, error, info};
use std::sync::Arc;

pub struct MarketDataCollector {
    exchange: Arc<Box<dyn Exchange>>,
    market_data_manager: Arc<MarketDataManager>,
    symbols: Vec<String>,
    shutdown_tx: broadcast::Sender<()>,
}

impl MarketDataCollector {
    pub fn new(
        exchange: Box<dyn Exchange>,
        market_data_manager: MarketDataManager,
        symbols: Vec<String>,
    ) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            exchange: Arc::new(exchange),
            market_data_manager: Arc::new(market_data_manager),
            symbols,
            shutdown_tx,
        }
    }
    
    pub async fn start(&self) -> Result<(), ExchangeError> {
        info!("Starting market data collection for symbols: {:?}", self.symbols);
        
        let market_data_manager = self.market_data_manager.clone();
        let callback = Box::new(move |data: MarketDataPoint| {
            let market_data_manager = market_data_manager.clone();
            tokio::spawn(async move {
                if let Err(e) = market_data_manager.store_market_data(&data).await {
                    error!("Failed to store market data: {}", e);
                }
            });
        });
        
        self.exchange.subscribe_market_data(&self.symbols, callback).await?;
        
        Ok(())
    }
    
    pub fn stop(&self) {
        info!("Stopping market data collection");
        let _ = self.shutdown_tx.send(());
    }
}

#[cfg(test)]
mod tests {
    use crate::services::exchange::binance::BinanceSpot;

    use super::*;
    use bigdecimal::Zero;
    use rust_decimal::Decimal;
    use tokio;
    
    #[tokio::test]
    async fn test_binance_ticker() {
        let exchange = BinanceSpot::new(None, None);
        let result = exchange.get_ticker("BTCUSDT").await;
        
        assert!(result.is_ok(), "Should successfully get ticker");
        
        if let Ok(ticker) = result {
            assert_eq!(ticker.symbol, "BTCUSDT");
            assert!(ticker.last_price > Decimal::zero());
            assert!(ticker.volume_24h > Decimal::zero());
        }
    }
    
    #[tokio::test]
    async fn test_binance_orderbook() {
        let exchange = BinanceSpot::new(None, None);
        let result = exchange.get_orderbook("BTCUSDT", 5).await;
        
        assert!(result.is_ok(), "Should successfully get orderbook");
        
        if let Ok(orderbook) = result {
            assert_eq!(orderbook.symbol, "BTCUSDT");
            assert!(!orderbook.bids.is_empty());
            assert!(!orderbook.asks.is_empty());
        }
    }
}