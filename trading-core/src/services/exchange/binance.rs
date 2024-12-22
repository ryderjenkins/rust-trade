use super::types::*;
use crate::data::market_data::MarketDataPoint;
use chrono::{DateTime, TimeZone, Utc};
use reqwest::{Client, Url};
use rust_decimal::Decimal;
use serde_json::Value;
use std::time::Duration;
use tokio_tungstenite::connect_async;
use tracing::{debug, error, info};
use futures_util::{SinkExt, StreamExt};  
use tokio_tungstenite::tungstenite::Message;  

#[derive(Clone)]
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
            ws_url: Url::parse("wss://stream.binance.com:9443").unwrap(),
            api_key,
            api_secret,
        }
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

    fn parse_ticker_message(&self, data: &serde_json::Value) -> Option<MarketDataPoint> {
        // 提取必要的字段
        let symbol = data.get("s")?.as_str()?;
        let price = data.get("c")?.as_str()?;
        let volume = data.get("v")?.as_str()?;
        let high = data.get("h")?.as_str()?;
        let low = data.get("l")?.as_str()?;
        let open = data.get("o")?.as_str()?;

        // 解析数据
        Some(MarketDataPoint {
            symbol: symbol.to_string(),
            price: price.parse().ok()?,
            volume: volume.parse().ok()?,
            timestamp: Utc::now(),
            high: high.parse().ok()?,
            low: low.parse().ok()?,
            open: open.parse().ok()?,
            close: price.parse().ok()?,
        })
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
                    price: kline[4].as_str().unwrap().parse().unwrap(),
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
        // 构建正确的 stream names
        let stream_names: Vec<String> = symbols
            .iter()
            .map(|s| format!("{}@ticker", s.to_lowercase()))
            .collect();

        // 正确构建 WebSocket URL，避免重复的 'ws' 路径
        let ws_url = if stream_names.len() == 1 {
            // 单个交易对格式：wss://stream.binance.com:9443/ws/btcusdt@ticker
            format!("wss://stream.binance.com:9443/ws/{}", stream_names[0])
        } else {
            // 多个交易对格式：wss://stream.binance.com:9443/stream?streams=btcusdt@ticker/ethusdt@ticker
            format!("wss://stream.binance.com:9443/stream?streams={}", stream_names.join("/"))
        };

        info!("Connecting to Binance WebSocket: {}", ws_url);

        // 建立 WebSocket 连接
        let (ws_stream, _response) = connect_async(&ws_url)
            .await
            .map_err(|e| ExchangeError::NetworkError(format!("WebSocket connection failed: {}", e)))?;

        info!("WebSocket connection established successfully");

        let (mut write, mut read) = ws_stream.split();

        // 对于多个交易对，发送订阅消息
        if stream_names.len() > 1 {
            let subscribe_msg = serde_json::json!({
                "method": "SUBSCRIBE",
                "params": stream_names,
                "id": 1
            });

            write
                .send(Message::Text(subscribe_msg.to_string()))
                .await
                .map_err(|e| ExchangeError::NetworkError(format!("Failed to send subscription: {}", e)))?;

            info!("Subscription message sent: {}", subscribe_msg);
        }

        // 处理接收到的消息
        while let Some(msg_result) = read.next().await {
            match msg_result {
                Ok(msg) => {
                    match msg {
                        Message::Text(text) => {
                            info!("Received market data: {}", text);
                            debug!("Received message: {}", text);
                            
                            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&text) {
                                // 处理市场数据
                                let ticker_data = if let Some(stream_data) = data.get("data") {
                                    stream_data // 多流格式
                                } else {
                                    &data // 单流格式
                                };

                                if let Some(market_data) = self.parse_ticker_message(ticker_data) {
                                    info!("Successfully parsed market data for {}: price={}", 
                                            market_data.symbol, market_data.price);
                                    callback(market_data);
                                }
                            }
                        }
                        Message::Ping(data) => {
                            write
                                .send(Message::Pong(data))
                                .await
                                .map_err(|e| ExchangeError::NetworkError(format!("Failed to send pong: {}", e)))?;
                        }
                        Message::Close(frame) => {
                            error!("WebSocket closed by server: {:?}", frame);
                            return Err(ExchangeError::NetworkError("Connection closed by server".into()));
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    return Err(ExchangeError::NetworkError(e.to_string()));
                }
            }
        }

        Ok(())
    }
}