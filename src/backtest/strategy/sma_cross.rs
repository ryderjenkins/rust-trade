// src/backtest/strategy/sma_cross.rs
use super::base::Strategy;
use crate::backtest::types::*;
use crate::data::market_data::MarketDataPoint;
use bigdecimal::{FromPrimitive, Zero};
use rust_decimal::Decimal;
use std::collections::VecDeque;

pub struct SimpleMovingAverageCrossStrategy {
    symbol: String,
    short_period: usize,
    long_period: usize,
    short_ma: VecDeque<f64>,
    long_ma: VecDeque<f64>,
    position_size: Decimal,
}

impl SimpleMovingAverageCrossStrategy {
    pub fn new(symbol: String, short_period: usize, long_period: usize, position_size: Decimal) -> Self {
        Self {
            symbol,
            short_period,
            long_period,
            short_ma: VecDeque::new(),
            long_ma: VecDeque::new(),
            position_size,
        }
    }

    fn calculate_ma(&mut self, price: f64) {
        self.short_ma.push_back(price);
        self.long_ma.push_back(price);

        if self.short_ma.len() > self.short_period {
            self.short_ma.pop_front();
        }
        if self.long_ma.len() > self.long_period {
            self.long_ma.pop_front();
        }
    }

    fn get_ma_values(&self) -> Option<(f64, f64)> {
        if self.short_ma.len() < self.short_period || self.long_ma.len() < self.long_period {
            return None;
        }

        let short_ma = self.short_ma.iter().sum::<f64>() / self.short_ma.len() as f64;
        let long_ma = self.long_ma.iter().sum::<f64>() / self.long_ma.len() as f64;

        Some((short_ma, long_ma))
    }
}

impl Strategy for SimpleMovingAverageCrossStrategy {
    fn on_data(&mut self, data: &MarketDataPoint, portfolio: &Portfolio) -> Vec<Order> {
        self.calculate_ma(data.price);

        let Some((short_ma, long_ma)) = self.get_ma_values() else {
            return vec![];
        };

        let mut orders = Vec::new();

        let price_decimal = Decimal::from_f64(data.price).unwrap_or_default();
        let quantity = if price_decimal > Decimal::zero() {
            self.position_size / price_decimal
        } else {
            Decimal::zero()
        };

        if short_ma > long_ma {
            // 生成买入信号
            if !portfolio.positions.contains_key(&self.symbol) {
                orders.push(Order {
                    symbol: self.symbol.clone(),
                    order_type: OrderType::Market,
                    side: OrderSide::Buy,
                    quantity,
                    timestamp: data.timestamp,
                });
            }
        } else if short_ma < long_ma {
            // 生成卖出信号
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

        orders
    }
}