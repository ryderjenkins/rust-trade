use super::types::*;
use bigdecimal::FromPrimitive;
use rust_decimal::Decimal;

pub struct RiskManager {
    max_position_size: Decimal,
    max_drawdown: Decimal,
    risk_per_trade: Decimal,
}

impl RiskManager {
    pub fn new(max_position_size: Decimal, max_drawdown: Decimal, risk_per_trade: Decimal) -> Self {
        Self {
            max_position_size,
            max_drawdown,
            risk_per_trade,
        }
    }

    pub fn validate_trade(&self, 
        order_size: Decimal, 
        current_portfolio_value: Decimal,
        analysis: &MarketAnalysis
    ) -> bool {
        // 检查订单大小
        if order_size > self.max_position_size {
            return false;
        }

        // 基于市场分析的风险评估
        match analysis.risk_level {
            RiskLevel::Extreme => false,
            RiskLevel::High => order_size <= self.max_position_size * Decimal::new(5, 1), // 50%
            RiskLevel::Medium => order_size <= self.max_position_size * Decimal::new(7, 1), // 70%
            RiskLevel::Low => true,
        }
    }

    pub fn calculate_position_size(&self, 
        signal_strength: f64,
        risk_level: &RiskLevel,
        portfolio_value: Decimal
    ) -> Decimal {
        let base_size = portfolio_value * self.risk_per_trade;
        
        // 根据信号强度和风险等级调整仓位大小
        let risk_multiplier = match risk_level {
            RiskLevel::Low => Decimal::new(1, 0),
            RiskLevel::Medium => Decimal::new(7, 1),
            RiskLevel::High => Decimal::new(5, 1),
            RiskLevel::Extreme => Decimal::new(2, 1),
        };

        let signal_strength_decimal = Decimal::from_f64(signal_strength)
            .unwrap_or(Decimal::new(5, 1));

        (base_size * risk_multiplier * signal_strength_decimal)
            .min(self.max_position_size)
    }
}