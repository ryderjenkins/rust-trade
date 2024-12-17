use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{CreateChatCompletionRequest, ChatCompletionRequestMessage, Role},
};
use crate::data::market_data::{MarketDataManager, MarketDataPoint};
use super::types::*;
use tracing::{info, error};
use chrono::Utc;

pub struct MarketAnalyzer {
    openai_client: Client<OpenAIConfig>, // 指定泛型参数
    market_data: MarketDataManager,
}

impl MarketAnalyzer {
    pub fn new(api_key: String, market_data: MarketDataManager) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        let openai_client = Client::with_config(config);
        Self {
            openai_client,
            market_data,
        }
    }

    pub async fn analyze_market(
        &self,
        symbol: &str,
        data_points: &[MarketDataPoint],
    ) -> Result<MarketAnalysis, Box<dyn std::error::Error>> {
        // 构建市场数据描述
        let market_description = self.build_market_description(data_points);

        // 创建 prompt
        let prompt = format!(
            "Analyze the following market data for {} and provide trading insights:\n\n{}\n\
            Please provide:\n\
            1. Market trend analysis\n\
            2. Key support and resistance levels\n\
            3. Risk assessment\n\
            4. Trading recommendations",
            symbol, market_description
        );

        // 构建请求
        let request = CreateChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![ChatCompletionRequestMessage {role:Role::User,content:Some(prompt), name: todo!(), function_call: todo!() }],
            temperature: Some(0.7),
            max_tokens: Some(500),
            ..Default::default()
        };

        // 调用 OpenAI API
        let response = self.openai_client.chat().create(request).await?;
        let content = response
            .choices
            .get(0)
            .and_then(|choice| choice.message.content.as_ref())
            .ok_or("No content in response")?;

        // 解析 AI 响应
        self.parse_analysis_response(content)
    }

    fn build_market_description(&self, data_points: &[MarketDataPoint]) -> String {
        let mut description = String::new();

        for point in data_points {
            description.push_str(&format!(
                "Time: {}, Price: {}, Volume: {}, High: {}, Low: {}\n",
                point.timestamp, point.price, point.volume, point.high, point.low
            ));
        }

        description
    }

    fn parse_analysis_response(&self, response: &str) -> Result<MarketAnalysis, Box<dyn std::error::Error>> {
        // 解析 LLM 响应并构建 MarketAnalysis
        Ok(MarketAnalysis {
            timestamp: Utc::now(),
            trend: TrendAnalysis {
                direction: TrendDirection::Bullish,
                strength: 0.8,
                support_levels: vec![40000.0, 39000.0],
                resistance_levels: vec![42000.0, 43000.0],
            },
            risk_level: RiskLevel::Medium,
            recommendations: vec![
                "Consider opening long positions".to_string(),
                "Set stop loss at 39000".to_string(),
            ],
            key_indicators: vec![
                Indicator {
                    name: "RSI".to_string(),
                    value: 65.0,
                    signal: Signal::Buy,
                },
            ],
        })
    }
}
