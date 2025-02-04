// trading-core/src/backtest/strategy/sma.rs
// This is only an example to test system function, we should not use it to trade. 

use crate::backtest::{Strategy, Order, Portfolio, OrderType, OrderSide};
use crate::data::market_data::MarketDataPoint;
use rust_decimal::Decimal;
use std::collections::{HashMap, VecDeque};

pub struct SMAStrategy {
    symbol: String,
    short_period: usize,
    long_period: usize,
    short_ma: VecDeque<f64>,
    long_ma: VecDeque<f64>,
    position_size: Decimal,
    parameters: HashMap<String, String>,
}

impl SMAStrategy {
    pub fn new(symbol: String, short_period: usize, long_period: usize, position_size: Decimal) -> Self {
        let mut parameters = HashMap::new();
        parameters.insert("short_period".to_string(), short_period.to_string());
        parameters.insert("long_period".to_string(), long_period.to_string());
        
        Self {
            symbol,
            short_period,
            long_period,
            short_ma: VecDeque::with_capacity(short_period),
            long_ma: VecDeque::with_capacity(long_period),
            position_size,
            parameters,
        }
    }

    fn calculate_ma(&mut self, price: f64) -> Option<(f64, f64)> {
        self.short_ma.push_back(price);
        self.long_ma.push_back(price);

        if self.short_ma.len() > self.short_period {
            self.short_ma.pop_front();
        }
        if self.long_ma.len() > self.long_period {
            self.long_ma.pop_front();
        }

        if self.short_ma.len() == self.short_period && self.long_ma.len() == self.long_period {
            let short_ma = self.short_ma.iter().sum::<f64>() / self.short_period as f64;
            let long_ma = self.long_ma.iter().sum::<f64>() / self.long_period as f64;
            Some((short_ma, long_ma))
        } else {
            None
        }
    }
}

impl Strategy for SMAStrategy {
    fn on_data(&mut self, data: &MarketDataPoint, portfolio: &Portfolio) -> Vec<Order> {
        let mut orders = Vec::new();
        
        // 计算移动平均线
        if let Some((short_ma, long_ma)) = self.calculate_ma(data.price) {
            // 生成交易信号
            if short_ma > long_ma {
                // 金叉，买入信号
                if !portfolio.positions.contains_key(&self.symbol) {
                    orders.push(Order {
                        symbol: self.symbol.clone(),
                        order_type: OrderType::Market,
                        side: OrderSide::Buy,
                        quantity: self.position_size,
                        timestamp: data.timestamp,
                    });
                }
            } else {
                // 死叉，卖出信号
                if let Some(position) = portfolio.positions.get(&self.symbol) {
                    orders.push(Order {
                        symbol: self.symbol.clone(),
                        order_type: OrderType::Market,
                        side: OrderSide::Sell,
                        quantity: position.quantity,
                        timestamp: data.timestamp,
                    });
                }
            }
        }
        
        orders
    }

    fn get_parameters(&self) -> &HashMap<String, String> {
        &self.parameters
    }

    fn get_type(&self) -> crate::backtest::StrategyType {
        crate::backtest::StrategyType::SMACross
    }
}