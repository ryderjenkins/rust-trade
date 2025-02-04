// trading-core/src/backtest/engine.rs

use super::{types::*, Strategy};
use super::metrics::MetricsCalculator;
use crate::data::market_data::{MarketDataManager, MarketDataPoint};
use bigdecimal::{FromPrimitive, Zero};
use chrono::{DateTime,Utc};
use rust_decimal::Decimal;
use std::{collections::HashMap, error::Error};
use tracing::{info, warn};

pub struct BacktestEngine {
    market_data: MarketDataManager,
    config: BacktestConfig,
    portfolio: Portfolio,
    trades: Vec<Trade>,
    metrics_calculator: MetricsCalculator,
    equity_points: Vec<EquityPoint>,
}

impl BacktestEngine {
    pub fn new(market_data: MarketDataManager, config: BacktestConfig) -> Self {
        let portfolio = Portfolio {
            cash: config.initial_capital,
            positions: HashMap::new(),
            total_value: config.initial_capital,
        };

        Self {
            market_data,
            config,
            portfolio,
            trades: Vec::new(),
            metrics_calculator: MetricsCalculator::new(),
            equity_points: Vec::new(),
        }
    }

    pub async fn run_strategy(
        &mut self,
        mut strategy: Box<dyn Strategy>,
    ) -> Result<BacktestResult, Box<dyn Error>> {
        info!("Starting backtest for symbol: {}", self.config.symbol);

        // 记录初始权益点
        self.record_equity_point(self.config.start_time, self.portfolio.total_value);

        let historical_data = self.market_data
            .get_market_data(
                &self.config.symbol,
                self.config.start_time,
                self.config.end_time,
            )
            .await?;

        info!("Loaded {} historical data points", historical_data.len());

        for data_point in historical_data {
            // 获取策略信号
            let orders = strategy.on_data(&data_point, &self.portfolio);
            
            // 执行订单
            for order in orders {
                if let Some(trade) = self.execute_order(&order, &data_point) {
                    info!("Executed trade: {} {} {} @ {}", 
                        trade.timestamp,
                        if trade.side == OrderSide::Buy { "BUY" } else { "SELL" },
                        trade.quantity,
                        trade.price
                    );
                    self.trades.push(trade);
                }
            }
            
            // 更新组合价值
            self.update_portfolio_value(&data_point);
            
            // 记录权益点
            self.record_equity_point(data_point.timestamp, self.portfolio.total_value);
        }

        info!("Backtest completed. Calculating metrics...");

        // 生成回测结果
        let metrics = self.metrics_calculator.calculate(
            &self.trades,
            &self.equity_points,
            &self.config
        );

        Ok(BacktestResult {
            strategy_type: strategy.get_type(),
            parameters: strategy.get_parameters().clone(),
            metrics,
            trades: self.trades.clone(),
            equity_curve: self.equity_points.clone(),
        })
    }

    fn execute_order(&mut self, order: &Order, data: &MarketDataPoint) -> Option<Trade> {
        let price = Decimal::from_f64(data.price)?;
        let commission = self.config.commission_rate * order.quantity * price;

        match order.side {
            OrderSide::Buy => {
                let cost = order.quantity * price + commission;
                if cost <= self.portfolio.cash {
                    self.portfolio.cash -= cost;
                    let position = self.portfolio.positions
                        .entry(order.symbol.clone())
                        .or_insert(Position {
                            symbol: order.symbol.clone(),
                            quantity: Decimal::zero(),
                            average_entry_price: Decimal::zero(),
                        });
                    
                    let new_quantity = position.quantity + order.quantity;
                    position.average_entry_price = 
                        (position.average_entry_price * position.quantity + price * order.quantity) 
                        / new_quantity;
                    position.quantity = new_quantity;

                    Some(Trade {
                        symbol: order.symbol.clone(),
                        side: OrderSide::Buy,
                        quantity: order.quantity,
                        price,
                        timestamp: data.timestamp,
                        commission,
                    })
                } else {
                    warn!("Insufficient funds for buy order");
                    None
                }
            },
            OrderSide::Sell => {
                if let Some(position) = self.portfolio.positions.get_mut(&order.symbol) {
                    if position.quantity >= order.quantity {
                        position.quantity -= order.quantity;
                        self.portfolio.cash += order.quantity * price - commission;
                        
                        if position.quantity.is_zero() {
                            self.portfolio.positions.remove(&order.symbol);
                        }

                        Some(Trade {
                            symbol: order.symbol.clone(),
                            side: OrderSide::Sell,
                            quantity: order.quantity,
                            price,
                            timestamp: data.timestamp,
                            commission,
                        })
                    } else {
                        warn!("Insufficient position for sell order");
                        None
                    }
                } else {
                    warn!("No position found for sell order");
                    None
                }
            }
        }
    }

    fn update_portfolio_value(&mut self, data: &MarketDataPoint) {
        let positions_value = self.portfolio.positions.values()
            .map(|pos| pos.quantity * Decimal::from_f64(data.price).unwrap_or_default())
            .sum::<Decimal>();

        self.portfolio.total_value = self.portfolio.cash + positions_value;
    }

    fn record_equity_point(&mut self, timestamp: DateTime<Utc>, value: Decimal) {
        self.equity_points.push(EquityPoint {
            timestamp: timestamp.to_rfc3339(),
            value: value.to_string(),
        });
    }
}