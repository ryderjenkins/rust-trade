use super::types::*;
use crate::data::market_data::MarketDataPoint;
use chrono::{DateTime, TimeZone, Utc};
use reqwest::{Client, Url};
use rust_decimal::Decimal;
use serde_json::Value;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tracing::{debug, error, info};
use futures_util::{SinkExt, StreamExt};  
use tokio_tungstenite::tungstenite::Message;  

pub struct BinanceSpot {
    client: Client,
    base_url: Url,
    ws_url: Url,
    api_key: Option<String>,
    api_secret: Option<String>,
}

impl BinanceSpot {
    pub fn new(api_key: Option<String>, api_secret: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            client,
            base_url: Url::parse("https://api.binance.com").unwrap(),
            ws_url: Url::parse("wss://stream.binance.com:9443/ws").unwrap(),  // 修正 WebSocket URL
            api_key,
            api_secret,
        }
    }

    async fn subscribe_market_data(
        &self,
        symbols: &[String],
        callback: Box<dyn Fn(MarketDataPoint) + Send + Sync>,
    ) -> Result<(), ExchangeError> {
        // 格式化交易对：把 BTCUSDT 转换为 btcusdt@ticker
        let symbol = symbols[0].to_lowercase();
        let ws_url = format!("wss://stream.binance.com:9443/ws/{}", symbol);
        
        debug!("Connecting to WebSocket URL: {}", ws_url);
    
        // 连接 WebSocket
        let (ws_stream, _) = connect_async(&ws_url)
            .await
            .map_err(|e| ExchangeError::NetworkError(e.to_string()))?;
            
        info!("WebSocket connected successfully");
        
        // 分离读写流
        let (write, mut read) = ws_stream.split();
    
        // 启动接收消息的任务
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(msg) => {
                        debug!("Received raw message: {}", msg);
                        if let Ok(data) = serde_json::from_str::<Value>(&msg.to_string()) {
                            // 提取价格数据
                            let market_data = MarketDataPoint {
                                timestamp: Utc::now(),
                                symbol: symbol.clone(),
                                price: data["p"].as_str()
                                    .unwrap_or("0")
                                    .parse()
                                    .unwrap_or(0.0),
                                volume: data["q"].as_str()
                                    .unwrap_or("0")
                                    .parse()
                                    .unwrap_or(0.0),
                                high: data["h"].as_str()
                                    .unwrap_or("0")
                                    .parse()
                                    .unwrap_or(0.0),
                                low: data["l"].as_str()
                                    .unwrap_or("0")
                                    .parse()
                                    .unwrap_or(0.0),
                                open: data["o"].as_str()
                                    .unwrap_or("0")
                                    .parse()
                                    .unwrap_or(0.0),
                                close: data["c"].as_str()
                                    .unwrap_or("0")
                                    .parse()
                                    .unwrap_or(0.0),
                            };
                            callback(market_data);
                        }
                    }
                    Err(e) => error!("WebSocket error: {}", e),
                }
            }
        });
    
        Ok(())
    }
    
    async fn make_request(&self, endpoint: &str, params: Option<Vec<(&str, String)>>) 
        -> Result<Value, ExchangeError> {
        let mut url = self.base_url.join(endpoint)
            .map_err(|e| ExchangeError::NetworkError(e.to_string()))?;
        
        if let Some(params) = params {
            let mut query = url.query_pairs_mut();
            for (key, value) in params {
                query.append_pair(key, &value);
            }
        }
        
        let mut request = self.client.get(url);
        if let Some(api_key) = &self.api_key {
            request = request.header("X-MBX-APIKEY", api_key);
        }
        
        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::NetworkError(e.to_string()))?;
            
        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ExchangeError::ApiError(error_text));
        }
        
        response.json::<Value>()
            .await
            .map_err(|e| ExchangeError::ApiError(e.to_string()))
    }
    
    fn parse_decimal(value: &str) -> Result<Decimal, ExchangeError> {
        value.parse()
            .map_err(|_| ExchangeError::ApiError("Invalid decimal format".to_string()))
    }
}

#[async_trait::async_trait]
impl Exchange for BinanceSpot {
    async fn get_ticker(&self, symbol: &str) -> Result<Ticker, ExchangeError> {
        let params = vec![("symbol", symbol.to_string())];
        let data = self.make_request("/api/v3/ticker/24hr", Some(params)).await?;
        
        Ok(Ticker {
            symbol: symbol.to_string(),
            timestamp: Utc::now(),
            last_price: Self::parse_decimal(data["lastPrice"].as_str().unwrap())?,
            bid_price: Self::parse_decimal(data["bidPrice"].as_str().unwrap())?,
            ask_price: Self::parse_decimal(data["askPrice"].as_str().unwrap())?,
            volume_24h: Self::parse_decimal(data["volume"].as_str().unwrap())?,
        })
    }
    
    async fn get_orderbook(&self, symbol: &str, limit: u32) -> Result<OrderBook, ExchangeError> {
        let params = vec![
            ("symbol", symbol.to_string()),
            ("limit", limit.to_string()),
        ];
        
        let data = self.make_request("/api/v3/depth", Some(params)).await?;
        
        let parse_levels = |levels: &Value| -> Result<Vec<OrderBookLevel>, ExchangeError> {
            levels.as_array()
                .ok_or_else(|| ExchangeError::ApiError("Invalid orderbook data".to_string()))?
                .iter()
                .map(|level| {
                    let price = Self::parse_decimal(level[0].as_str().unwrap())?;
                    let quantity = Self::parse_decimal(level[1].as_str().unwrap())?;
                    Ok(OrderBookLevel { price, quantity })
                })
                .collect()
        };
        
        Ok(OrderBook {
            symbol: symbol.to_string(),
            timestamp: Utc::now(),
            bids: parse_levels(&data["bids"])?,
            asks: parse_levels(&data["asks"])?,
        })
    }
    
    async fn get_recent_trades(&self, symbol: &str, limit: u32) -> Result<Vec<Trade>, ExchangeError> {
        let params = vec![
            ("symbol", symbol.to_string()),
            ("limit", limit.to_string()),
        ];
        
        let data = self.make_request("/api/v3/trades", Some(params)).await?;
        
        data.as_array()
            .ok_or_else(|| ExchangeError::ApiError("Invalid trades data".to_string()))?
            .iter()
            .map(|trade| {
                Ok(Trade {
                    symbol: symbol.to_string(),
                    timestamp: Utc.timestamp_millis_opt(trade["time"].as_i64().unwrap()).unwrap(),
                    price: Self::parse_decimal(trade["price"].as_str().unwrap())?,
                    quantity: Self::parse_decimal(trade["qty"].as_str().unwrap())?,
                    is_buyer_maker: trade["isBuyerMaker"].as_bool().unwrap(),
                })
            })
            .collect()
    }
    
    async fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<u32>,
    ) -> Result<Vec<MarketDataPoint>, ExchangeError> {
        let mut params = vec![
            ("symbol", symbol.to_string()),
            ("interval", interval.to_string()),
        ];
        
        if let Some(start) = start_time {
            params.push(("startTime", start.timestamp_millis().to_string()));
        }
        if let Some(end) = end_time {
            params.push(("endTime", end.timestamp_millis().to_string()));
        }
        if let Some(limit) = limit {
            params.push(("limit", limit.to_string()));
        }
        
        let data = self.make_request("/api/v3/klines", Some(params)).await?;
        
        data.as_array()
            .ok_or_else(|| ExchangeError::ApiError("Invalid kline data".to_string()))?
            .iter()
            .map(|kline| {
                Ok(MarketDataPoint {
                    timestamp: Utc.timestamp_millis_opt(kline[0].as_i64().unwrap()).unwrap(),
                    symbol: symbol.to_string(),
                    price: kline[4].as_str().unwrap().parse().unwrap(),  // 收盘价作为当前价格
                    volume: kline[5].as_str().unwrap().parse().unwrap(),
                    high: kline[2].as_str().unwrap().parse().unwrap(),
                    low: kline[3].as_str().unwrap().parse().unwrap(),
                    open: kline[1].as_str().unwrap().parse().unwrap(),
                    close: kline[4].as_str().unwrap().parse().unwrap(),
                })
            })
            .collect()
    }
    
    async fn subscribe_market_data(
        &self,
        symbols: &[String],
        callback: Box<dyn Fn(MarketDataPoint) + Send + Sync>,
    ) -> Result<(), ExchangeError> {
        let streams: Vec<String> = symbols
            .iter()
            .map(|symbol| format!("{}@ticker", symbol.to_lowercase()))
            .collect();
            
        let ws_url = format!("{}/stream?streams={}", self.ws_url, streams.join("/"));
        
        let (ws_stream, _) = connect_async(ws_url)
            .await
            .map_err(|e| ExchangeError::NetworkError(e.to_string()))?;
            
        let (write, mut read) = ws_stream.split();
        
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(msg) => {
                        if let Ok(data) = serde_json::from_str::<Value>(&msg.to_string()) {
                            if let Some(ticker) = data.get("data") {
                                let symbol = ticker["s"].as_str().unwrap().to_string();
                                let market_data = MarketDataPoint {
                                    timestamp: Utc::now(),
                                    symbol: symbol,
                                    price: ticker["c"].as_str().unwrap().parse().unwrap(),
                                    volume: ticker["v"].as_str().unwrap().parse().unwrap(),
                                    high: ticker["h"].as_str().unwrap().parse().unwrap(),
                                    low: ticker["l"].as_str().unwrap().parse().unwrap(),
                                    open: ticker["o"].as_str().unwrap().parse().unwrap(),
                                    close: ticker["c"].as_str().unwrap().parse().unwrap(),
                                };
                                callback(market_data);
                            }
                        }
                    }
                    Err(e) => error!("WebSocket error: {}", e),
                }
            }
        });
        
        Ok(())
    }
}