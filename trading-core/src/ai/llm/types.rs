// ai/llm/types.rs
use serde::Serialize;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize)]
pub struct MarketAnalysis {
    pub timestamp: DateTime<Utc>,
    pub trend: TrendAnalysis,
    pub risk_level: RiskLevel,
    pub recommendations: Vec<String>,
    pub key_indicators: Vec<Indicator>,
}

#[derive(Debug, Serialize)]
pub struct TrendAnalysis {
    pub direction: TrendDirection,
    pub strength: f64,
    pub support_levels: Vec<f64>,
    pub resistance_levels: Vec<f64>,
}

#[derive(Debug, Serialize)]
pub enum TrendDirection {
    Bullish,
    Bearish,
    Sideways,
}

#[derive(Debug, Serialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Extreme,
}

#[derive(Debug, Serialize)]
pub struct Indicator {
    pub name: String,
    pub value: f64,
    pub signal: Signal,
}

#[derive(Debug, Serialize)]
pub enum Signal {
    Buy,
    Sell,
    Neutral,
}