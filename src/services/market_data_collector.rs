use crate::data::market_data::{MarketDataManager, MarketDataPoint};
use crate::services::exchange::types::{Exchange, ExchangeError};
use tokio::sync::{broadcast, mpsc};
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};
use std::sync::Arc;
use dotenv::dotenv;

const RECONNECT_DELAY: Duration = Duration::from_secs(5);
const CHANNEL_BUFFER_SIZE: usize = 1000;

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
        
        // 创建数据通道
        let (data_tx, mut data_rx) = mpsc::channel::<MarketDataPoint>(CHANNEL_BUFFER_SIZE);
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        
        // 克隆需要的变量用于异步任务
        let exchange = self.exchange.clone();
        let symbols = self.symbols.clone();
        let market_data_manager = self.market_data_manager.clone();
        
        // 启动WebSocket订阅任务
        let subscription_handle = tokio::spawn(async move {
            loop {
                let callback = {
                    let data_tx = data_tx.clone();
                    Box::new(move |data: MarketDataPoint| {
                        let data_tx = data_tx.clone();
                        tokio::spawn(async move {
                            if let Err(e) = data_tx.send(data).await {
                                error!("Failed to send market data through channel: {}", e);
                            }
                        });
                    })
                };
                
                match exchange.subscribe_market_data(&symbols, callback).await {
                    Ok(()) => {
                        info!("Successfully subscribed to market data");
                    }
                    Err(e) => {
                        error!("Failed to subscribe to market data: {}", e);
                        sleep(RECONNECT_DELAY).await;
                        continue;
                    }
                }
                
                // 等待关闭信号或重连
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        info!("Received shutdown signal, stopping subscription");
                        break;
                    }
                    _ = sleep(Duration::from_secs(60)) => {
                        warn!("WebSocket connection timeout, reconnecting...");
                        continue;
                    }
                }
            }
        });
        
        // 启动数据处理任务
        let processing_handle = tokio::spawn(async move {
            while let Some(data) = data_rx.recv().await {
                match market_data_manager.store_market_data(&data).await {
                    Ok(()) => {
                        debug!(
                            "Stored market data: symbol={}, price={}, volume={}", 
                            data.symbol, data.price, data.volume
                        );
                    }
                    Err(e) => {
                        error!("Failed to store market data: {}", e);
                    }
                }
            }
        });
        
        // 等待任务完成
        tokio::try_join!(subscription_handle, processing_handle)
            .map_err(|e| ExchangeError::NetworkError(e.to_string()))?;
        
        Ok(())
    }
    
    pub fn stop(&self) {
        info!("Stopping market data collection");
        let _ = self.shutdown_tx.send(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::database::Database;
    use crate::services::exchange::binance::BinanceSpot;
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_market_data_collection() {
        // 加载环境变量
        dotenv().ok();

        tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_test_writer()
        .init();

        info!("Starting market data collection test");

        // 设置测试数据库连接
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set for tests");
            
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create database pool");
            
        // 创建市场数据管理器
        let market_data_manager = MarketDataManager::new(pool);

        info!("Initializing Binance connection...");
        
        // 创建交易所服务
        let exchange = BinanceSpot::new(None, None);
        
        // 创建数据采集器
        let collector = Arc::new(MarketDataCollector::new(
            Box::new(exchange),
            market_data_manager,
            vec!["btcusdt@ticker".to_string()],
        ));

        info!("Starting data collection...");
        
        let collector_clone = collector.clone();
        let handle = tokio::spawn(async move {
            info!("Collector task started");
            collector_clone.start().await.expect("Failed to start collector");
            info!("Collector task finished");
        });
        
        // 运行一段时间后停止
        info!("Waiting for 10 seconds to collect data...");
        tokio::time::sleep(Duration::from_secs(10)).await;
        info!("Stopping collector...");
        collector.stop();
        
        // 等待采集器停止
        handle.await.expect("Collector task failed");
        info!("Test completed successfully");
    }
}