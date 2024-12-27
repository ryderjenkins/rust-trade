use super::base::Strategy;
use crate::backtest::types::*;
use crate::data::market_data::MarketDataPoint;
use bigdecimal::{FromPrimitive, Zero};
use rust_decimal::Decimal;
use std::collections::VecDeque;
use tracing::{debug, info};

pub struct SimpleMovingAverageCrossStrategy {
    symbol: String,
    short_period: usize,
    long_period: usize,
    short_ma: VecDeque<f64>,
    long_ma: VecDeque<f64>,
    position_size: Decimal,
    last_signal: Option<bool>,
}

impl SimpleMovingAverageCrossStrategy {
    pub fn new(symbol: String, short_period: usize, long_period: usize, position_size: Decimal) -> Self {
        info!(
            "Initializing SMA Cross Strategy - Symbol: {}, Short Period: {}, Long Period: {}, Position Size: {}",
            symbol, short_period, long_period, position_size
        );
        
        Self {
            symbol,
            short_period,
            long_period,
            short_ma: VecDeque::with_capacity(long_period),
            long_ma: VecDeque::with_capacity(long_period),
            position_size,
            last_signal: None,
        }
    }

    fn calculate_ma(&mut self, price: f64) -> Option<(f64, f64)> {
        // 添加价格到两个MA队列
        self.short_ma.push_back(price);
        self.long_ma.push_back(price);

        // 维护队列大小
        if self.short_ma.len() > self.short_period {
            self.short_ma.pop_front();
        }
        if self.long_ma.len() > self.long_period {
            self.long_ma.pop_front();
        }

        info!(
            "Price: {}, Short MA Length: {}/{}, Long MA Length: {}/{}",
            price,
            self.short_ma.len(),
            self.short_period,
            self.long_ma.len(),
            self.long_period
        );

        // 只要短期均线有效就开始生成信号
        if self.short_ma.len() == self.short_period {
            let short_ma = self.short_ma.iter().sum::<f64>() / self.short_ma.len() as f64;
            let long_ma = if self.long_ma.len() >= self.long_period {
                self.long_ma.iter().sum::<f64>() / self.long_ma.len() as f64
            } else {
                self.long_ma.iter().sum::<f64>() / self.long_ma.len() as f64
            };

            info!("MA Values - Short: {}, Long: {}", short_ma, long_ma);
            Some((short_ma, long_ma))
        } else {
            None
        }
    }
}

impl Strategy for SimpleMovingAverageCrossStrategy {
    fn on_data(&mut self, data: &MarketDataPoint, portfolio: &Portfolio) -> Vec<Order> {
        let Some((short_ma, long_ma)) = self.calculate_ma(data.price) else {
            return vec![];
        };

        let mut orders = Vec::new();
        let price_decimal = Decimal::from_f64(data.price).unwrap_or_default();
        let quantity = if price_decimal > Decimal::zero() {
            self.position_size / price_decimal
        } else {
            Decimal::zero()
        };

        let current_signal = short_ma > long_ma;
        let signal_changed = self.last_signal.map_or(true, |last| last != current_signal);

        info!(
            "Signal Analysis - Price: {}, Short MA: {}, Long MA: {}, Signal: {}, Position: {}",
            data.price,
            short_ma,
            long_ma,
            if current_signal { "BUY" } else { "SELL" },
            portfolio.positions.contains_key(&self.symbol)
        );

        if current_signal && signal_changed {
            // 买入信号
            if !portfolio.positions.contains_key(&self.symbol) {
                info!("Generating BUY order at price: {}", price_decimal);
                orders.push(Order {
                    symbol: self.symbol.clone(),
                    order_type: OrderType::Market,
                    side: OrderSide::Buy,
                    quantity,
                    timestamp: data.timestamp,
                });
            }
        } else if !current_signal && signal_changed {
            // 卖出信号
            if let Some(position) = portfolio.positions.get(&self.symbol) {
                info!("Generating SELL order at price: {}", price_decimal);
                orders.push(Order {
                    symbol: self.symbol.clone(),
                    order_type: OrderType::Market,
                    side: OrderSide::Sell,
                    quantity: position.quantity,
                    timestamp: data.timestamp,
                });
            }
        }

        self.last_signal = Some(current_signal);
        orders
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_sma_strategy() {
        let mut strategy = SimpleMovingAverageCrossStrategy::new(
            "BTCUSDT".to_string(),
            5,
            10,
            Decimal::new(100, 0), // 100 units position size
        );

        let mut portfolio = Portfolio {
            cash: Decimal::new(10000, 0),
            positions: Default::default(),
            total_value: Decimal::new(10000, 0),
        };

        // 模拟上升趋势的价格序列
        let prices = vec![
            100.0, 101.0, 102.0, 103.0, 104.0,  // 建立短期均线
            105.0, 106.0, 107.0, 108.0, 109.0,  // 建立长期均线
            110.0, 111.0, 112.0, 113.0, 114.0,  // 上升趋势
        ];

        for price in prices {
            let data = MarketDataPoint {
                timestamp: Utc::now(),
                symbol: "BTCUSDT".to_string(),
                price,
                volume: 1.0,
                high: price,
                low: price,
                open: price,
                close: price,
            };

            let orders = strategy.on_data(&data, &portfolio);
            println!("Price: {}, Orders generated: {}", price, orders.len());
        }
    }
}