// trading-core/src/backtest/types.rs

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// 基础配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub initial_capital: Decimal,
    pub symbol: String,
    pub commission_rate: Decimal,
}

// 策略类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyType {
    SMACross,
    RSI,
    MACD,
    BollingerBands,
    Custom(String),
}

// 订单类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit(Decimal),
}

// 订单方向
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

// 订单结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub symbol: String,
    pub order_type: OrderType,
    pub side: OrderSide,
    pub quantity: Decimal,
    pub timestamp: DateTime<Utc>,
}

// 交易结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: Decimal,
    pub price: Decimal,
    pub timestamp: DateTime<Utc>,
    pub commission: Decimal,
}

// 持仓信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub quantity: Decimal,
    pub average_entry_price: Decimal,
}

// 投资组合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub cash: Decimal,
    pub positions: HashMap<String, Position>,
    pub total_value: Decimal,
}

// 权益点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityPoint {
    pub timestamp: String,
    pub value: String,
}

// 回测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub strategy_type: StrategyType,
    pub parameters: HashMap<String, String>,
    pub metrics: Metrics,
    pub trades: Vec<Trade>,
    pub equity_curve: Vec<EquityPoint>,
}

// 性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    // 基础指标
    pub total_return: Decimal,
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub win_rate: Decimal,
    pub profit_factor: Decimal,
    
    // 风险指标
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub max_drawdown: Decimal,
    pub max_drawdown_duration: i64,  // 以秒为单位
    
    // 收益指标
    pub avg_profit_per_trade: Decimal,
    pub avg_winning_trade: Decimal,
    pub avg_losing_trade: Decimal,
    pub largest_winning_trade: Decimal,
    pub largest_losing_trade: Decimal,
    
    // 交易指标
    pub avg_trade_duration: i64,     // 以秒为单位
    pub profit_per_month: Decimal,
    pub annual_return: Decimal,
    pub monthly_sharpe: f64,
    
    // 额外统计
    pub total_commission: Decimal,
    pub total_volume: Decimal,
    pub avg_position_size: Decimal,
}

// 前端请求结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestRequest {
    pub strategy_type: StrategyType,
    pub parameters: HashMap<String, String>,
    pub config: BacktestConfig,
}

// 前端响应结构
#[derive(Serialize)]
pub struct TradeResponse {
    pub timestamp: String,
    pub symbol: String,
    pub side: String,
    pub quantity: String,
    pub price: String,
    pub commission: String,
}

#[derive(Serialize)]
pub struct BacktestResponse {
    pub total_return: String,
    pub sharpe_ratio: f64,
    pub max_drawdown: String,
    pub win_rate: String,
    pub total_trades: u32,
    pub equity_curve: Vec<EquityPoint>,
    pub trades: Vec<TradeResponse>,
}

// 策略评分结果（为 NFT 准备）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyScore {
    pub total_score: u32,
    pub return_score: u32,
    pub risk_score: u32,
    pub consistency_score: u32,
    pub uniqueness_score: u32,
    pub rating: StrategyRating,
}

// 策略评级（为 NFT 准备）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyRating {
    Legendary,
    Epic,
    Rare,
    Common,
}

// NFT 元数据（为 NFT 准备）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyNFTMetadata {
    pub strategy_id: String,
    pub name: String,
    pub description: String,
    pub creator: String,
    pub creation_date: DateTime<Utc>,
    pub metrics: Metrics,
    pub score: StrategyScore,
    pub parameters: HashMap<String, String>,
    pub trading_period: String,
    pub symbol: String,
    pub image_url: Option<String>,
}