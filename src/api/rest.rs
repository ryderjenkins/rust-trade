use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Router,
};
use std::sync::Arc;
use crate::services::exchange::types::{Exchange, ExchangeError};
use super::types::*;
use crate::data::market_data::MarketDataPoint;

pub struct ApiContext {
    pub exchange: Arc<Box<dyn Exchange>>,
}

pub fn create_router(context: Arc<ApiContext>) -> Router {
    Router::new()
        .route("/api/v1/market/ticker/:symbol", axum::routing::get(get_ticker))
        .route("/api/v1/market/orderbook/:symbol", axum::routing::get(get_orderbook))
        .route("/api/v1/market/klines", axum::routing::get(get_klines))
        .with_state(context)
}

async fn get_ticker(
    State(context): State<Arc<ApiContext>>,
    Path(symbol): Path<String>,
) -> Result<Json<ApiResponse<MarketTickerResponse>>, StatusCode> {
    match context.exchange.get_ticker(&symbol).await {
        Ok(ticker) => {
            let response = MarketTickerResponse {
                symbol: ticker.symbol,
                price: ticker.last_price,
                timestamp: ticker.timestamp,
                volume_24h: ticker.volume_24h,
                high_24h: ticker.bid_price, 
                low_24h: ticker.ask_price,
            };
            
            Ok(Json(ApiResponse {
                success: true,
                data: Some(response),
                error: None,
            }))
        }
        Err(e) => {
            Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }))
        }
    }
}

async fn get_orderbook(
    State(context): State<Arc<ApiContext>>,
    Path(symbol): Path<String>,
    Query(limit): Query<Option<u32>>,
) -> Result<Json<ApiResponse<OrderBookResponse>>, StatusCode> {
    let limit = limit.unwrap_or(20);
    
    match context.exchange.get_orderbook(&symbol, limit).await {
        Ok(orderbook) => {
            let response = OrderBookResponse {
                symbol: orderbook.symbol,
                timestamp: orderbook.timestamp,
                bids: orderbook.bids.into_iter()
                    .map(|level| (level.price, level.quantity))
                    .collect(),
                asks: orderbook.asks.into_iter()
                    .map(|level| (level.price, level.quantity))
                    .collect(),
            };
            
            Ok(Json(ApiResponse {
                success: true,
                data: Some(response),
                error: None,
            }))
        }
        Err(e) => {
            Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }))
        }
    }
}

async fn get_klines(
    State(context): State<Arc<ApiContext>>,
    Query(query): Query<KlineQuery>,
) -> Result<Json<ApiResponse<Vec<MarketDataPoint>>>, StatusCode> {
    match context.exchange.get_klines(
        &query.symbol,
        &query.interval,
        query.start_time,
        query.end_time,
        query.limit,
    ).await {
        Ok(klines) => {
            Ok(Json(ApiResponse {
                success: true,
                data: Some(klines),
                error: None,
            }))
        }
        Err(e) => {
            Ok(Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }))
        }
    }
}