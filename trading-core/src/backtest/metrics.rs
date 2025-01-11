// trading-core/src/backtest/metrics.rs

use super::types::*;
use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use std::collections::HashMap;

pub struct MetricsCalculator {
    risk_free_rate: f64,
}

impl MetricsCalculator {
    pub fn new() -> Self {
        Self {
            risk_free_rate: 0.02,
        }
    }

    pub fn calculate(
        &self,
        trades: &[Trade],
        equity_points: &[EquityPoint],
        config: &BacktestConfig,
    ) -> Metrics {
        let _ = config;
        let (profit_trades, loss_trades) = self.analyze_trades(trades);
        let returns = self.calculate_returns(equity_points);
        let (max_drawdown, max_drawdown_duration) = self.calculate_drawdown(equity_points);

        Metrics {
            // 基础指标 - 已实现
            total_return: self.calculate_total_return(equity_points),
            total_trades: trades.len() as u32,
            winning_trades: profit_trades.len() as u32,
            losing_trades: loss_trades.len() as u32,
            win_rate: self.calculate_win_rate(profit_trades.len(), trades.len()),
            profit_factor: self.calculate_profit_factor(&profit_trades, &loss_trades),
            
            // 风险指标 - 已实现
            sharpe_ratio: self.calculate_sharpe_ratio(&returns),
            sortino_ratio: self.calculate_sortino_ratio(&returns),
            max_drawdown,
            max_drawdown_duration: max_drawdown_duration.num_seconds(),
            
            // 交易统计 - 已实现
            avg_profit_per_trade: self.calculate_avg_profit(trades),
            total_commission: trades.iter().map(|t| t.commission).sum(),
            total_volume: self.calculate_total_volume(trades),
            
            // TODO: 待实现的指标
            avg_winning_trade: Decimal::zero(),  // 需要实现
            avg_losing_trade: Decimal::zero(),   // 需要实现
            largest_winning_trade: Decimal::zero(), // 需要实现
            largest_losing_trade: Decimal::zero(),  // 需要实现
            avg_trade_duration: 0,               // 需要实现
            profit_per_month: Decimal::zero(),   // 需要实现
            annual_return: Decimal::zero(),      // 需要实现
            monthly_sharpe: 0.0,                 // 需要实现
            avg_position_size: Decimal::zero(),  // 需要实现
        }
    }

    fn analyze_trades(&self, trades: &[Trade]) -> (Vec<Trade>, Vec<Trade>) {
        let mut profit_trades = Vec::new();
        let mut loss_trades = Vec::new();
        let mut position_map: HashMap<String, (Decimal, Decimal)> = HashMap::new();

        for trade in trades {
            match trade.side {
                OrderSide::Buy => {
                    let (qty, avg_price) = position_map
                        .entry(trade.symbol.clone())
                        .or_insert((Decimal::zero(), Decimal::zero()));
                    
                    *avg_price = (*avg_price * *qty + trade.price * trade.quantity) 
                        / (*qty + trade.quantity);
                    *qty += trade.quantity;
                }
                OrderSide::Sell => {
                    if let Some((_, avg_price)) = position_map.get(&trade.symbol) {
                        if trade.price > *avg_price {
                            profit_trades.push(trade.clone()); 
                        } else {
                            loss_trades.push(trade.clone()); 
                        }
                    }
                }
            }
        }

        (profit_trades, loss_trades)
    }

    fn calculate_returns(&self, equity_points: &[EquityPoint]) -> Vec<f64> {
        equity_points.windows(2)
            .map(|window| {
                let prev_value = Decimal::from_str(&window[0].value).unwrap_or_default();
                let curr_value = Decimal::from_str(&window[1].value).unwrap_or_default();
                if prev_value.is_zero() {
                    0.0
                } else {
                    ((curr_value - prev_value) / prev_value).to_f64().unwrap_or_default()
                }
            })
            .collect()
    }

    fn calculate_drawdown(&self, equity_points: &[EquityPoint]) -> (Decimal, Duration) {
        let mut max_drawdown = Decimal::zero();
        let mut max_drawdown_duration = Duration::zero();
        let mut peak_value = Decimal::zero();
        let mut peak_time = Utc::now();
        
        for point in equity_points {
            let value = Decimal::from_str(&point.value).unwrap_or_default();
            let time = DateTime::parse_from_rfc3339(&point.timestamp)
                .unwrap_or_else(|_| Utc::now().into())
                .with_timezone(&Utc);

            if value > peak_value {
                peak_value = value;
                peak_time = time;
            } else if peak_value > Decimal::zero() {
                let drawdown = (peak_value - value) / peak_value;
                let duration = time - peak_time;
                
                if drawdown > max_drawdown {
                    max_drawdown = drawdown;
                    max_drawdown_duration = duration;
                }
            }
        }
        
        (max_drawdown, max_drawdown_duration)
    }

    fn calculate_total_return(&self, equity_points: &[EquityPoint]) -> Decimal {
        if equity_points.len() < 2 {
            return Decimal::zero();
        }

        let initial_value = Decimal::from_str(&equity_points[0].value).unwrap_or_default();
        let final_value = Decimal::from_str(&equity_points[equity_points.len() - 1].value).unwrap_or_default();

        if initial_value.is_zero() {
            return Decimal::zero();
        }

        ((final_value - initial_value) / initial_value) * Decimal::from(100)
    }

    fn calculate_win_rate(&self, winning_trades: usize, total_trades: usize) -> Decimal {
        if total_trades == 0 {
            return Decimal::zero();
        }
        
        Decimal::from(winning_trades) / Decimal::from(total_trades) * Decimal::from(100)
    }

    fn calculate_profit_factor(&self, profit_trades: &[Trade], loss_trades: &[Trade]) -> Decimal {
        let total_profit = profit_trades.iter()
            .map(|t| (t.price - t.commission) * t.quantity)
            .sum::<Decimal>();

        let total_loss = loss_trades.iter()
            .map(|t| (t.price + t.commission) * t.quantity)
            .sum::<Decimal>();

        if total_loss.is_zero() {
            return if total_profit.is_zero() { Decimal::one() } else { Decimal::MAX };
        }

        total_profit / total_loss
    }

    fn calculate_sharpe_ratio(&self, returns: &[f64]) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let volatility = returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>()
            .sqrt() * (252.0_f64).sqrt();

        if volatility == 0.0 {
            return 0.0;
        }

        (mean_return * 252.0 - self.risk_free_rate) / volatility
    }

    fn calculate_sortino_ratio(&self, returns: &[f64]) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let downside_returns: Vec<f64> = returns.iter()
            .filter(|&&r| r < 0.0)
            .map(|&r| r.powi(2))
            .collect();

        if downside_returns.is_empty() {
            return 0.0;
        }

        let downside_deviation = (downside_returns.iter().sum::<f64>() / downside_returns.len() as f64).sqrt() 
            * (252.0_f64).sqrt();

        if downside_deviation == 0.0 {
            return 0.0;
        }

        (mean_return * 252.0 - self.risk_free_rate) / downside_deviation
    }

    fn calculate_avg_profit(&self, trades: &[Trade]) -> Decimal {
        if trades.is_empty() {
            return Decimal::zero();
        }

        let total_profit = trades.iter()
            .map(|t| t.price * t.quantity - t.commission)
            .sum::<Decimal>();

        total_profit / Decimal::from(trades.len())
    }

    fn calculate_total_volume(&self, trades: &[Trade]) -> Decimal {
        trades.iter()
            .map(|t| t.quantity * t.price)
            .sum()
    }
}

// TODO: 待实现的辅助函数
// fn calculate_avg_winning_trade()
// fn calculate_avg_losing_trade()
// fn find_largest_profit()
// fn find_largest_loss()
// fn calculate_avg_trade_duration()
// fn calculate_monthly_profit()
// fn calculate_annual_return()
// fn calculate_monthly_sharpe()
// fn calculate_avg_position_size()