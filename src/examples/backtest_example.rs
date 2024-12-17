use crate::data::market_data::MarketDataManager;
use crate::backtest::{
    engine::engine::BacktestEngine,
    strategy::sma_cross::SimpleMovingAverageCrossStrategy,
    types::BacktestConfig,
};
use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::str::FromStr;

pub async fn run_backtest_example(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建市场数据管理器
    let market_data = MarketDataManager::new(pool);

    // 2. 设置回测配置
    let config = BacktestConfig {
        start_time: Utc::now() - Duration::days(30),
        end_time: Utc::now(),
        initial_capital: Decimal::from_str("10000.0")?,
        symbol: "BTCUSDT".to_string(),
        commission_rate: Decimal::from_str("0.001")?,
    };

    // 3. 创建策略实例
    let position_size = Decimal::from_str("1000.0")?;
    let mut strategy = SimpleMovingAverageCrossStrategy::new(
        config.symbol.clone(),
        5,
        20,
        position_size,
    );

    // 4. 创建回测引擎
    let mut engine = BacktestEngine::new(market_data, config);

    // 5. 运行回测
    println!("开始回测...");
    let result = engine.run(&mut strategy).await?;

    // 6. 输出结果
    println!("\n回测结果:");
    println!("总收益率: {}%", result.total_return);
    println!("总交易次数: {}", result.total_trades);
    println!("盈利交易: {}", result.winning_trades);
    println!("亏损交易: {}", result.losing_trades);
    println!("最大回撤: {}%", result.max_drawdown);

    println!("\n交易记录:");
    for (i, trade) in result.trades.iter().enumerate() {
        println!(
            "{}. {} {} {} @ {} (手续费: {})",
            i + 1,
            trade.timestamp,
            match trade.side {
                crate::backtest::types::OrderSide::Buy => "买入",
                crate::backtest::types::OrderSide::Sell => "卖出",
            },
            trade.quantity,
            trade.price,
            trade.commission
        );
    }

    Ok(())
}