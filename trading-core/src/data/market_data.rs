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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TickData {
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
    pub side: String,
    pub trade_id: String,
    pub is_maker: bool,
}

#[derive(Clone)]
pub struct MarketDataManager {
    pool: PgPool,
}

impl MarketDataManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    pub fn get_pool(&self) -> PgPool {
        self.pool.clone()
    }

    // 存储市场数据现在实际上是存储tick数据
    pub async fn store_market_data(
        &self,
        data: &MarketDataPoint,
    ) -> Result<(), MarketDataError> {
        debug!("Storing tick data for symbol: {}", data.symbol);
        
        sqlx::query!(
            r#"
            INSERT INTO tick_data 
            (timestamp, symbol, price, volume, side, trade_id, is_maker)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            data.timestamp,
            data.symbol,
            data.price,
            data.volume,
            "BUY",  // 默认使用BUY，因为我们没有方向信息
            format!("auto_{}", Utc::now().timestamp_nanos()),
            false
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to store tick data: {}", e);
            MarketDataError::DatabaseError(e)
        })?;

        debug!("Tick data stored successfully");
        Ok(())
    }

    // 获取市场数据现在需要聚合tick数据
    pub async fn get_market_data(
        &self,
        symbol: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<MarketDataPoint>, MarketDataError> {
        debug!("Fetching market data for symbol: {}", symbol);
        
        // 直接获取所有 tick 数据，不进行时间聚合
        let rows = sqlx::query!(
            r#"
            SELECT 
                timestamp as "timestamp!",
                symbol as "symbol!",
                price as "price!",
                volume as "volume!",
                price as "high!",  -- 对于 tick 数据，价格即为 OHLC
                price as "low!",
                price as "open!",
                price as "close!"
            FROM tick_data
            WHERE symbol = $1 
            AND timestamp >= $2 
            AND timestamp <= $3
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

        info!("Fetched {} tick data points", rows.len());
    
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
            FROM tick_data
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
            FROM tick_data
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
        info!("Cleaning up tick data older than {} days", days_to_keep);
        
        let result = sqlx::query!(
            r#"
            WITH deleted AS (
                DELETE FROM tick_data
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
        info!("Cleaned up {} old tick data records", deleted_count);
        Ok(deleted_count)
    }

    pub async fn get_candlestick_data(
        &self,
        symbol: &str,
        interval: &str,
        start_time: Option<chrono::NaiveDateTime>,
        end_time: Option<chrono::NaiveDateTime>,
    ) -> Result<Vec<MarketDataPoint>, MarketDataError> {
        debug!(
            "Fetching candlestick data for symbol: {}, interval: {}",
            symbol, interval
        );
    
        let rows = sqlx::query!(
            r#"
            WITH time_slots AS (
                SELECT 
                    date_trunc($4, timestamp) as slot_time,
                    first_value(price) OVER w as open,
                    max(price) OVER w as high,
                    min(price) OVER w as low,
                    last_value(price) OVER w as close,
                    sum(volume) OVER w as volume
                FROM tick_data
                WHERE symbol = $1
                AND ($2::timestamp IS NULL OR timestamp >= $2)
                AND ($3::timestamp IS NULL OR timestamp <= $3)
                WINDOW w AS (
                    PARTITION BY date_trunc($4, timestamp)
                    ORDER BY timestamp
                    ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING
                )
            )
            SELECT DISTINCT
                slot_time as "timestamp!",
                $1 as "symbol!",
                close as "price!",
                volume as "volume!",
                high as "high!",
                low as "low!",
                open as "open!",
                close as "close!"
            FROM time_slots
            ORDER BY slot_time DESC
            "#,
            symbol,
            start_time,
            end_time,
            interval
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch candlestick data: {}", e);
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

        // 清理旧数据
        sqlx::query!("DELETE FROM tick_data WHERE symbol = $1", test_data.symbol)
            .execute(&manager.pool)
            .await
            .expect("Failed to clean up old test data");

        // 存储数据
        manager.store_market_data(&test_data)
            .await
            .expect("Failed to store market data");

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // 验证数据
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
    }
}