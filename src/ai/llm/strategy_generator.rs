use async_openai::config::OpenAIConfig;
use async_openai::types::Role;
use tracing::{error, info};
use super::risk::RiskManager;
use super::types::*;
use crate::backtest::{types::*, Strategy};
use crate::data::market_data::MarketDataPoint;
use rust_decimal::Decimal;
use std::sync::mpsc;
use async_openai::{Client, types::{CreateChatCompletionRequest, ChatCompletionRequestMessage}};

pub struct LLMStrategy {
    symbol: String,
    risk_manager: RiskManager,
    position_size: Decimal,
    api_key: String,
    has_analyzed: bool,
    last_signal: Option<OrderSide>,
    sender: mpsc::Sender<String>,
    receiver: mpsc::Receiver<String>,
}

impl LLMStrategy {
    pub fn new(
        symbol: String,
        api_key: String,
        risk_manager: RiskManager,
        position_size: Decimal,
    ) -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            symbol,
            api_key,
            risk_manager,
            position_size,
            has_analyzed: false,
            last_signal: None,
            sender,
            receiver,
        }
    }

    fn analyze_first_data(&mut self, data: &MarketDataPoint) {
        let api_key = self.api_key.clone();
        let sender = self.sender.clone();
        let symbol = self.symbol.clone();
        let data = data.clone(); // Clone the data if necessary
        
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let config = OpenAIConfig::new().with_api_key(api_key);
                let client = Client::with_config(config);
    
                // Create the prompt
                let prompt = format!(
                    "Based on this single market data point for {}, give me a ONE WORD trading signal (only say BUY, SELL, or HOLD):\n\
                     Price: ${:.2}\n\
                     Volume: {:.2}\n\
                     24h High: ${:.2}\n\
                     24h Low: ${:.2}\n",
                    symbol, data.price, data.volume, data.high, data.low
                );
    
                let request = CreateChatCompletionRequest {
                    model: "gpt-3.5-turbo".into(),
                    messages: vec![ChatCompletionRequestMessage {
                        role: Role::User,
                        content: Some(prompt),
                        name: None,
                        function_call: None,
                    }],
                    temperature: Some(0.3),
                    max_tokens: Some(10),
                    ..Default::default()
                };
    
                match client.chat().create(request).await {
                    Ok(response) => {
                        if let Some(choice) = response.choices.first() {
                            if let Some(content) = &choice.message.content {
                                // Output the LLM result here
                                info!("LLM Signal: {}", content);
    
                                // Send the result to the channel for further use
                                let _ = sender.send(content.clone());
                            }
                        }
                    }
                    Err(e) => error!("API call failed: {}", e),
                }
            });
        });
    }    
}

impl Strategy for LLMStrategy {
    fn on_data(&mut self, data: &MarketDataPoint, portfolio: &Portfolio) -> Vec<Order> {
        let mut orders = Vec::new();

        // 只分析第一个数据点
        if !self.has_analyzed {
            info!("Requesting LLM analysis for first data point...");
            self.analyze_first_data(data);
            self.has_analyzed = true;
        }

        // 检查是否有分析结果
        if let Ok(signal) = self.receiver.try_recv() {
            info!("Received LLM signal: {}", signal);
            let action = if signal.to_uppercase().contains("BUY") {
                Some(OrderSide::Buy)
            } else if signal.to_uppercase().contains("SELL") {
                Some(OrderSide::Sell)
            } else {
                None
            };

            if let Some(side) = action {
                match side {
                    OrderSide::Buy if self.last_signal != Some(OrderSide::Buy) => {
                        orders.push(Order {
                            symbol: self.symbol.clone(),
                            order_type: OrderType::Market,
                            side: OrderSide::Buy,
                            quantity: self.position_size,
                            timestamp: data.timestamp,
                        });
                        self.last_signal = Some(OrderSide::Buy);
                        info!("Generated BUY order for {}", self.symbol);
                    }
                    OrderSide::Sell if self.last_signal != Some(OrderSide::Sell) => {
                        if let Some(position) = portfolio.positions.get(&self.symbol) {
                            orders.push(Order {
                                symbol: self.symbol.clone(),
                                order_type: OrderType::Market,
                                side: OrderSide::Sell,
                                quantity: position.quantity,
                                timestamp: data.timestamp,
                            });
                            self.last_signal = Some(OrderSide::Sell);
                            info!("Generated SELL order for {}", self.symbol);
                        }
                    }
                    _ => {}
                }
            }
        }

        orders
    }
}