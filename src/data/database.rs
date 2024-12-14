use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing::info;
use crate::config::Database as DbConfig;

pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new(config: &DbConfig) -> Result<Self, sqlx::Error> {
        info!("Initializing database connection pool...");
        
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .max_lifetime(std::time::Duration::from_secs(config.max_lifetime))
            .connect(&config.url)
            .await?;

        info!("Database connection pool initialized successfully");
        Ok(Self { pool })
    }

    pub async fn check_connection(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        info!("Database connection test successful");
        Ok(())
    }
}