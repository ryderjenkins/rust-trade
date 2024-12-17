# Backtesting System Documentation

## Overview
The backtesting system is designed to test trading strategies using historical market data. It provides a flexible framework for strategy development, order execution simulation, and performance analysis.

## Architecture

```
backtest/
├── types.rs              // Core data structures
├── engine/               // Backtesting engine
│   ├── engine.rs         // Core backtesting logic
│   └── executor.rs       // Order execution simulator
└── strategy/             // Strategy framework
    ├── base.rs           // Strategy trait definition
    └── sma_cross.rs      // Sample SMA crossover strategy
```

## Core Components

### 1. Strategy Interface
The `Strategy` trait defines the interface for implementing trading strategies:

```rust
pub trait Strategy: Send + Sync {
    fn on_data(&mut self, data: &MarketDataPoint, portfolio: &Portfolio) -> Vec<Order>;
}
```

### 2. Backtesting Engine
The engine simulates market conditions and executes trading orders:

```rust
pub struct BacktestEngine {
    market_data: MarketDataManager,
    config: BacktestConfig,
    portfolio: Portfolio,
    trades: Vec<Trade>,
    executor: OrderExecutor,
}
```

### 3. Order Executor
Simulates order execution with realistic conditions:
- Market order execution
- Commission calculation
- Position tracking
- Portfolio value updates

## Configuration

### BacktestConfig
```rust
pub struct BacktestConfig {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub initial_capital: Decimal,
    pub symbol: String,
    pub commission_rate: Decimal,
}
```

## Usage

### 1. Command Line Interface
Run backtesting through command line:

```bash
# Default parameters
cargo run -- backtest

# Custom parameters
cargo run -- backtest \
    --symbol BTCUSDT \
    --days 30 \
    --initial-capital 10000.0 \
    --commission-rate 0.001 \
    --short-period 5 \
    --long-period 20
```

### 2. Programmatic Usage
```rust
// 1. Create market data manager
let market_data = MarketDataManager::new(pool);

// 2. Configure backtest parameters
let config = BacktestConfig {
    start_time: start_time,
    end_time: end_time,
    initial_capital: Decimal::from_str("10000.0")?,
    symbol: "BTCUSDT".to_string(),
    commission_rate: Decimal::from_str("0.001")?,
};

// 3. Initialize strategy
let strategy = SimpleMovingAverageCrossStrategy::new(
    config.symbol.clone(),
    5,  // short period
    20, // long period
    position_size,
);

// 4. Create and run backtest engine
let mut engine = BacktestEngine::new(market_data, config);
let result = engine.run(&mut strategy).await?;
```

## Implemented Strategies

### 1. Simple Moving Average Crossover
- Uses two moving averages (short and long period)
- Generates buy signals when short MA crosses above long MA
- Generates sell signals when short MA crosses below long MA

```rust
let strategy = SimpleMovingAverageCrossStrategy::new(
    "BTCUSDT".to_string(),
    5,    // short period
    20,   // long period
    position_size,
);
```

## Backtest Results
The system provides comprehensive backtest results:

```rust
pub struct BacktestResult {
    pub total_return: Decimal,    // Total return percentage
    pub total_trades: u32,        // Number of trades executed
    pub winning_trades: u32,      // Number of profitable trades
    pub losing_trades: u32,       // Number of losing trades
    pub max_drawdown: Decimal,    // Maximum drawdown percentage
    pub trades: Vec<Trade>,       // Detailed trade history
}
```

## Example Output
```
Backtest Results:
Total Return: 15.7%
Total Trades: 24
Winning Trades: 14
Losing Trades: 10
Maximum Drawdown: 8.3%

Trade History:
1. 2024-01-15 10:30:00 BUY 0.5 @ 42156.85 (fee: 21.08)
2. 2024-01-16 15:45:00 SELL 0.5 @ 43100.00 (fee: 21.55)
...
```

## Best Practices

1. **Data Preparation**
   - Ensure sufficient historical data
   - Verify data quality and consistency
   - Consider market hours and trading volume

2. **Strategy Development**
   - Start with simple strategies
   - Test with different parameters
   - Consider transaction costs
   - Account for market conditions

3. **Performance Analysis**
   - Evaluate multiple metrics
   - Consider risk-adjusted returns
   - Analysis across different market conditions

## Limitations and Considerations

1. **Market Impact**
   - The system assumes perfect execution
   - Does not account for slippage
   - May not reflect actual market impact

2. **Data Quality**
   - Results depend on historical data quality
   - Missing data points may affect results
   - Consider market hours and holidays

3. **Strategy Constraints**
   - Limited to single symbol testing
   - No support for limit orders yet
   - Basic position sizing

## Future Enhancements

1. **Performance Metrics**
   - Sharpe Ratio
   - Sortino Ratio
   - Risk-adjusted returns
   - Drawdown analysis

2. **Strategy Framework**
   - Multiple symbol support
   - Advanced order types
   - Risk management rules
   - Position sizing strategies

3. **Visualization**
   - Equity curve plotting
   - Trade entry/exit points
   - Performance metrics charts
   - Drawdown visualization