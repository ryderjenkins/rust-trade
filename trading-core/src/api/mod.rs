pub mod types;
pub mod rest;

use axum::serve;
use std::net::SocketAddr;
use std::sync::Arc;
use crate::services::exchange::types::Exchange; 
use tokio::net::TcpListener;

pub struct ApiServer {
    exchange: Arc<Box<dyn Exchange>>,
    addr: SocketAddr,
}

impl ApiServer {
    pub fn new(exchange: Box<dyn Exchange>, addr: SocketAddr) -> Self {
        Self {
            exchange: Arc::new(exchange),
            addr,
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let context = Arc::new(rest::ApiContext {
            exchange: self.exchange.clone(),
        });

        let app = rest::create_router(context);
        
        println!("API server listening on {}", self.addr);
        
        let listener = TcpListener::bind(&self.addr).await?;
        serve(listener, app).await?;

        Ok(())
    }
}