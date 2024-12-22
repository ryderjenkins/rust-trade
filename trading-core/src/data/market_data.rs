// data/market_data.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use tracing::{debug, error, info};

#[derive(Error, Debug)]
pub enum MarketDataError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Invalid data format: {0}")]
    InvalidDataFormat(String),
    #[error("Data fetch error: {0}")]
    FetchError(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketDataPoint {
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
    pub high: f64,
    pub low: f64,
    pub open: f64,
    pub close: f64,
}

impl MarketDataPoint {
    pub fn new(
        timestamp: DateTime<Utc>,
        symbol: String,
        price: f64,
        volume: f64,
        high: f64,
        low: f64,
        open: f64,
        close: f64,
    ) -> Self {
        Self {
            timestamp,
            symbol,
            price,
            volume,
            high,
            low,
            open,
            close,
        }
    }
}

#[derive(Clone)]
pub struct MarketDataManager {
    pool: PgPool,
}

impl MarketDataManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn store_market_data(
        &self,
        data: &MarketDataPoint,
    ) -> Result<(), MarketDataError> {
        debug!("Storing market data for symbol: {}", data.symbol);
        
        sqlx::query!(
            r#"
            INSERT INTO market_data 
            (timestamp, symbol, price, volume, high, low, open, close)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            data.timestamp,
            data.symbol,
            data.price,
            data.volume,
            data.high,
            data.low,
            data.open,
            data.close
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to store market data: {}", e);
            MarketDataError::DatabaseError(e)
        })?;

        debug!("Market data stored successfully");
        Ok(())
    }

    pub async fn get_market_data(
        &self,
        symbol: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<MarketDataPoint>, MarketDataError> {
        debug!("Fetching market data for symbol: {}", symbol);
        
        let rows = sqlx::query!(
            r#"
            SELECT 
                timestamp as "timestamp!", 
                symbol as "symbol!", 
                price as "price!", 
                volume as "volume!", 
                high as "high!", 
                low as "low!", 
                open as "open!", 
                close as "close!"
            FROM market_data
            WHERE symbol = $1 AND timestamp >= $2 AND timestamp <= $3
            ORDER BY timestamp ASC
            "#,
            symbol,
            start_time,
            end_time
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch market data: {}", e);
            MarketDataError::DatabaseError(e)
        })?;
    
        Ok(rows
            .into_iter()
            .map(|row| MarketDataPoint {
                timestamp: row.timestamp,
                symbol: row.symbol,
                price: row.price,
                volume: row.volume,
                high: row.high,
                low: row.low,
                open: row.open,
                close: row.close,
            })
            .collect())
    }
    
    pub async fn get_latest_price(&self, symbol: &str) -> Result<f64, MarketDataError> {
        debug!("Fetching latest price for symbol: {}", symbol);
        
        let row = sqlx::query!(
            r#"
            SELECT price as "price!"
            FROM market_data
            WHERE symbol = $1
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
            symbol
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch latest price: {}", e);
            MarketDataError::DatabaseError(e)
        })?;
    
        Ok(row.price)
    }
    
    pub async fn calculate_vwap(
        &self,
        symbol: &str,
        window_minutes: f64,
    ) -> Result<f64, MarketDataError> {
        debug!("Calculating VWAP for symbol: {} with window: {} minutes", symbol, window_minutes);
        
        let row = sqlx::query!(
            r#"
            SELECT COALESCE(
                CAST(SUM(price * volume) / NULLIF(SUM(volume), 0) AS DOUBLE PRECISION),
                0.0
            ) as "vwap!"
            FROM market_data
            WHERE symbol = $1 
            AND timestamp >= NOW() - INTERVAL '1 minute' * $2
            "#,
            symbol,
            window_minutes
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to calculate VWAP: {}", e);
            MarketDataError::DatabaseError(e)
        })?;
    
        Ok(row.vwap)
    }
    
    pub async fn cleanup_old_data(
        &self,
        days_to_keep: f64,
    ) -> Result<u64, MarketDataError> {
        info!("Cleaning up market data older than {} days", days_to_keep);
        
        let result = sqlx::query!(
            r#"
            WITH deleted AS (
                DELETE FROM market_data
                WHERE timestamp < NOW() - INTERVAL '1 day' * $1
                RETURNING *
            )
            SELECT COUNT(*) as "count!"
            FROM deleted
            "#,
            days_to_keep
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to cleanup old data: {}", e);
            MarketDataError::DatabaseError(e)
        })?;
    
        let deleted_count = result.count as u64;
        info!("Cleaned up {} old market data records", deleted_count);
        Ok(deleted_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use sqlx::postgres::PgPoolOptions;
    use dotenv::dotenv;

    async fn setup_test_db() -> PgPool {

        dotenv().ok();
        
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set for tests");
            
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create test database pool")
    }

    #[tokio::test]
    async fn test_market_data_operations() {
        let pool = setup_test_db().await;
        let manager = MarketDataManager::new(pool);

        // 使用固定时间来避免时区问题
        let timestamp = Utc::now();
        let test_data = MarketDataPoint::new(
            timestamp,
            "BTC/USDT".to_string(), 
            50000.0,
            1.5,
            51000.0,
            49000.0,
            49500.0,
            50000.0,
        );

        // 清理可能存在的旧数据
        sqlx::query!("DELETE FROM market_data WHERE symbol = $1", test_data.symbol)
            .execute(&manager.pool)
            .await
            .expect("Failed to clean up old test data");

        // 存储数据
        manager.store_market_data(&test_data)
            .await
            .expect("Failed to store market data");

        // 确保数据已经存储
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // 查询并验证数据
        let start_time = timestamp - Duration::hours(1);
        let end_time = timestamp + Duration::hours(1);
        let retrieved_data = manager
            .get_market_data(&test_data.symbol, start_time, end_time)
            .await
            .expect("Failed to retrieve market data");

        assert!(!retrieved_data.is_empty(), "Retrieved data should not be empty");
        
        let first_item = &retrieved_data[0];
        assert_eq!(first_item.symbol, test_data.symbol, "Symbol mismatch");
        assert_eq!(first_item.price, test_data.price, "Price mismatch");
        assert_eq!(first_item.volume, test_data.volume, "Volume mismatch");
        assert_eq!(first_item.high, test_data.high, "High mismatch");
        assert_eq!(first_item.low, test_data.low, "Low mismatch");
        assert_eq!(first_item.open, test_data.open, "Open mismatch");
        assert_eq!(first_item.close, test_data.close, "Close mismatch");

        // 测试获取最新价格
        let latest_price = manager
            .get_latest_price(&test_data.symbol)
            .await
            .expect("Failed to get latest price");
        assert_eq!(latest_price, test_data.price, "Latest price mismatch");

        // 清理测试数据
        sqlx::query!("DELETE FROM market_data WHERE symbol = $1", test_data.symbol)
            .execute(&manager.pool)
            .await
            .expect("Failed to clean up test data");
    }

    // 添加其他测试
    #[tokio::test]
    async fn test_vwap_calculation() {
        let pool = setup_test_db().await;
        let manager = MarketDataManager::new(pool);

        // 插入测试数据
        let symbol = "ETH/USDT";
        let timestamp = Utc::now();
        
        let test_data = vec![
            MarketDataPoint::new(
                timestamp,
                symbol.to_string(),
                2000.0,
                1.0,
                2000.0,
                2000.0,
                2000.0,
                2000.0,
            ),
            MarketDataPoint::new(
                timestamp + Duration::minutes(1),
                symbol.to_string(),
                2100.0,
                2.0,
                2100.0,
                2100.0,
                2100.0,
                2100.0,
            ),
        ];

        // 清理旧数据
        sqlx::query!("DELETE FROM market_data WHERE symbol = $1", symbol)
            .execute(&manager.pool)
            .await
            .expect("Failed to clean up old test data");

        // 插入测试数据
        for data in &test_data {
            manager.store_market_data(data)
                .await
                .expect("Failed to store market data");
        }

        // 计算预期 VWAP: (2000*1 + 2100*2)/(1 + 2) = 2066.67
        let expected_vwap = (2000.0 * 1.0 + 2100.0 * 2.0) / (1.0 + 2.0);
        
        let vwap = manager.calculate_vwap(symbol, 5.0)
            .await
            .expect("Failed to calculate VWAP");

        // 使用近似相等来比较浮点数
        assert!((vwap - expected_vwap).abs() < 0.01, 
            "VWAP calculation mismatch: expected {}, got {}", expected_vwap, vwap);

        // 清理测试数据
        sqlx::query!("DELETE FROM market_data WHERE symbol = $1", symbol)
            .execute(&manager.pool)
            .await
            .expect("Failed to clean up test data");
    }
}