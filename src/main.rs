mod config;
mod data;
mod services;

use dotenv::dotenv;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载 .env 文件
    dotenv().ok();

    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    info!("Starting Rust Trade System...");

    let settings = config::Settings::new()?;
    info!("Configuration loaded successfully");
    info!("Database URL: {}", settings.database.url);
    info!("API Port: {}", settings.api.port);

    // 初始化数据库连接
    let database = data::database::Database::new(&settings.database).await?;

    // 测试数据库连接
    database.check_connection().await?;
    
    Ok(())
}