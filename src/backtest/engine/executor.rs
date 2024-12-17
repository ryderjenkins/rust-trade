use super::super::types::*;
use crate::data::market_data::MarketDataPoint;
use bigdecimal::{FromPrimitive, Zero};
use rust_decimal::Decimal;

pub struct OrderExecutor {
    commission_rate: Decimal,
}

impl OrderExecutor {
    pub fn new(commission_rate: Decimal) -> Self {
        Self { commission_rate }
    }

    pub fn execute_order(
        &self,
        order: &Order,
        data: &MarketDataPoint,
        portfolio: &mut Portfolio,
    ) -> Option<Trade> {
        match order.side {
            OrderSide::Buy => self.execute_buy(order, data, portfolio),
            OrderSide::Sell => self.execute_sell(order, data, portfolio),
        }
    }

    fn execute_buy(
        &self,
        order: &Order,
        data: &MarketDataPoint,
        portfolio: &mut Portfolio,
    ) -> Option<Trade> {
        let price = Decimal::from_f64(data.price).unwrap_or_default();
        let total_cost = price * order.quantity;
        let commission = total_cost * self.commission_rate;

        if total_cost + commission > portfolio.cash {
            return None;
        }

        portfolio.cash -= total_cost + commission;
        
        let position = portfolio.positions
            .entry(order.symbol.clone())
            .or_insert(Position {
                symbol: order.symbol.clone(),
                quantity: Decimal::zero(),
                average_entry_price: Decimal::zero(),
            });

        let new_total = position.quantity * position.average_entry_price + order.quantity * price;
        position.quantity += order.quantity;
        position.average_entry_price = new_total / position.quantity;

        Some(Trade {
            symbol: order.symbol.clone(),
            side: OrderSide::Buy,
            quantity: order.quantity,
            price,
            timestamp: order.timestamp,
            commission,
        })
    }

    fn execute_sell(
        &self,
        order: &Order,
        data: &MarketDataPoint,
        portfolio: &mut Portfolio,
    ) -> Option<Trade> {
        let position = match portfolio.positions.get_mut(&order.symbol) {
            Some(pos) if pos.quantity >= order.quantity => pos,
            _ => return None,
        };

        let price = Decimal::from_f64(data.price).unwrap_or_default();
        let total_value = price * order.quantity;
        let commission = total_value * self.commission_rate;

        portfolio.cash += total_value - commission;
        position.quantity -= order.quantity;

        if position.quantity == Decimal::zero() {
            portfolio.positions.remove(&order.symbol);
        }

        Some(Trade {
            symbol: order.symbol.clone(),
            side: OrderSide::Sell,
            quantity: order.quantity,
            price,
            timestamp: order.timestamp,
            commission,
        })
    }
}