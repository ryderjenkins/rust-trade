mod config;
mod data;
mod services;
mod api;

use dotenv::dotenv;
use tracing::{info, Level};
use std::net::SocketAddr;
use tokio::signal;
use crate::api::ApiServer;
use crate::data::database::Database;
use crate::services::exchange::binance::BinanceSpot;
use crate::config::Settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载 .env 文件
    dotenv().ok();

    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // 加载配置
    let settings = Settings::new()?;
    info!("Configuration loaded successfully");

    // 初始化数据库连接
    let database = Database::new(&settings.database).await?;
    database.check_connection().await?;
    info!("Database connection established");

    // 初始化交易所服务
    let exchange = BinanceSpot::new(None, None);
    info!("Exchange service initialized");

    // 启动 API 服务器
    let addr = SocketAddr::from(([127, 0, 0, 1], settings.api.port));
    let api_server = ApiServer::new(Box::new(exchange), addr);
    
    info!("Starting API server on {}", addr);
    
    // 关闭处理
    let shutdown_signal = async {
        signal::ctrl_c()
            .await
            .expect("Failed to initialize shutdown signal handler");
        info!("Shutdown signal received");
    };

    // 使用 tokio::select! 来处理服务器运行和关闭信号
    tokio::select! {
        result = api_server.run() => {
            if let Err(e) = result {
                tracing::error!("API server error: {}", e);
            }
        }
        _ = shutdown_signal => {
            info!("Shutting down API server");
        }
    }

    info!("Server shutdown complete");
    Ok(())
}