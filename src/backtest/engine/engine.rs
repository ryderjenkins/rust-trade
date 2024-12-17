use crate::data::market_data::{MarketDataManager, MarketDataPoint};
use crate::backtest::strategy::base::Strategy;
use crate::backtest::types::*;
use rust_decimal::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use tracing::info;

use super::executor::OrderExecutor;

pub struct BacktestEngine {
    market_data: MarketDataManager,
    config: BacktestConfig,
    portfolio: Portfolio,
    trades: Vec<Trade>,
    executor: OrderExecutor,
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
            executor: OrderExecutor::new(config.commission_rate),
            config,
            portfolio,
            trades: Vec::new(),
        }
    }

    pub async fn run(&mut self, strategy: &mut dyn Strategy) -> Result<BacktestResult, Box<dyn Error>> {
        info!("Starting backtest for {} from {} to {}", 
            self.config.symbol, 
            self.config.start_time, 
            self.config.end_time
        );

        let historical_data = self.market_data
            .get_market_data(
                &self.config.symbol,
                self.config.start_time,
                self.config.end_time,
            )
            .await?;

        info!("Loaded {} historical data points", historical_data.len());

        for data_point in historical_data {
            let orders = strategy.on_data(&data_point, &self.portfolio);
            
            for order in orders {
                if let Some(trade) = self.executor.execute_order(&order, &data_point, &mut self.portfolio) {
                    info!("Executed trade: {} {} {} @ {}", 
                        trade.timestamp,
                        if matches!(trade.side, OrderSide::Buy) { "BUY" } else { "SELL" },
                        trade.quantity,
                        trade.price
                    );
                    self.trades.push(trade);
                }
            }
            
            self.update_portfolio_value(&data_point);
        }

        let result = self.generate_result();
        info!("Backtest completed. Total return: {}%, Total trades: {}", 
            result.total_return,
            result.total_trades
        );

        Ok(result)
    }

    fn update_portfolio_value(&mut self, data: &MarketDataPoint) {
        let positions_value = self.portfolio.positions.values()
            .map(|pos| pos.quantity * Decimal::from_f64(data.price).unwrap_or_default())
            .sum::<Decimal>();

        self.portfolio.total_value = self.portfolio.cash + positions_value;
    }

    fn generate_result(&self) -> BacktestResult {
        let (winning_trades, losing_trades) = self.calculate_trade_statistics();
        
        let return_pct = if self.config.initial_capital > Decimal::zero() {
            (self.portfolio.total_value - self.config.initial_capital) 
                / self.config.initial_capital * Decimal::from(100)
        } else {
            Decimal::zero()
        };

        BacktestResult {
            total_return: return_pct,
            total_trades: self.trades.len() as u32,
            winning_trades,
            losing_trades,
            max_drawdown: self.calculate_max_drawdown(),
            trades: self.trades.clone(),
        }
    }

    fn calculate_trade_statistics(&self) -> (u32, u32) {
        let mut winning = 0;
        let mut losing = 0;

        for trade in &self.trades {
            match trade.side {
                OrderSide::Sell => {
                    if let Some(position) = self.portfolio.positions.get(&trade.symbol) {
                        if trade.price > position.average_entry_price {
                            winning += 1;
                        } else {
                            losing += 1;
                        }
                    }
                }
                _ => {}
            }
        }

        (winning, losing)
    }

    fn calculate_max_drawdown(&self) -> Decimal {
        let mut max_drawdown = Decimal::zero();
        let mut peak = self.config.initial_capital;

        // 这里最大回撤细节需要完善
        for _ in &self.trades {
            let current_value = self.portfolio.total_value;
            if current_value > peak {
                peak = current_value;
            } else if peak > Decimal::zero() {
                let drawdown = (peak - current_value) / peak * Decimal::from(100);
                if drawdown > max_drawdown {
                    max_drawdown = drawdown;
                }
            }
        }
        max_drawdown
    }
}