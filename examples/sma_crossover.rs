use backtesting::data::DataLoader;
use backtesting::indicators::{Indicator, SimpleMovingAverage};
use backtesting::{Order, OrderSide, OrderType};
use backtesting::types::OHLCV;
use backtesting::{Backtest, BacktestConfig, Strategy};

struct SMACrossover {
    fast_sma: SimpleMovingAverage,
    slow_sma: SimpleMovingAverage,
    position: bool,
}

impl SMACrossover {
    fn new(fast_period: usize, slow_period: usize) -> Self {
        Self {
            fast_sma: SimpleMovingAverage::new(fast_period),
            slow_sma: SimpleMovingAverage::new(slow_period),
            position: false,
        }
    }
}

impl Strategy for SMACrossover {
    fn init(&mut self, _data: &[OHLCV]) -> backtesting::Result<()> {
        println!("Initializing SMA Crossover strategy");
        Ok(())
    }

    fn next(&mut self, bar: &OHLCV, _index: usize) -> backtesting::Result<Vec<Order>> {
        // Update indicators
        let fast_sma = self.fast_sma.update(bar);
        let slow_sma = self.slow_sma.update(bar);

        // Check if we have valid signals
        match (fast_sma, slow_sma) {
            (Some(fast), Some(slow)) => {
                let mut orders = Vec::with_capacity(1);

                // Buy signal: fast SMA crosses above slow SMA
                if fast > slow && !self.position {
                    self.position = true;
                    orders.push(Order::new(
                        OrderSide::Buy,
                        OrderType::Market,
                        50.0,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ));
                    println!(
                        "BUY signal at {:.2} (Fast: {:.2}, Slow: {:.2})",
                        bar.close, fast, slow
                    );
                }
                // Sell signal: fast SMA crosses below slow SMA
                else if fast < slow && self.position {
                    self.position = false;
                    orders.push(Order::new(
                        OrderSide::Sell,
                        OrderType::Market,
                        50.0,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ));
                    println!(
                        "SELL signal at {:.2} (Fast: {:.2}, Slow: {:.2})",
                        bar.close, fast, slow
                    );
                }

                Ok(orders)
            }
            _ => Ok(vec![]),
        }
    }
}
fn main() -> backtesting::Result<()> {
    println!("Running SMA Crossover backtest example");

    // Create sample data
    let data = DataLoader::load_from_csv("sample")?;

    for (i, bar) in data.iter().enumerate() {
        println!(
            "Day {}: {} - OHLCV({:.2}, {:.2}, {:.2}, {:.2}, {:.0})",
            i + 1,
            bar.timestamp.format("%Y-%m-%d"),
            bar.open,
            bar.high,
            bar.low,
            bar.close,
            bar.volume
        );
    }

    // Configure backtest
    let config = BacktestConfig {
        initial_cash: 10000.0,
        commission: 0.001, // 0.1% commission
        ..Default::default()
    };

    // Create strategy with 20/50 SMA crossover
    let strategy = SMACrossover::new(20, 50);

    // Create and run backtest
    let mut backtest = Backtest::new(&data, config);
    let results = backtest.run(strategy)?;

    // Print results
    println!("\n=== SMA Crossover Backtest Results ===");
    println!("Start Date: {}", results.start_date.format("%Y-%m-%d"));
    println!("End Date: {}", results.end_date.format("%Y-%m-%d"));
    println!("Duration: {} days", results.duration.num_days());
    println!();

    // Performance metrics
    println!("=== Performance ===");
    println!("Initial Cash: ${:.2}", 10000.0);
    println!("Final Equity: ${:.2}", results.equity_final);
    println!("Equity Peak: ${:.2}", results.equity_peak);
    println!("Total Return: {:.2}%", results.return_pct * 100.0);
    println!("Annualized Return: {:.2}%", results.return_ann * 100.0);
    println!(
        "Buy & Hold Return: {:.2}%",
        results.buy_hold_return_pct * 100.0
    );
    println!("Exposure Time: {:.2}%", results.exposure_time * 100.0);
    println!();

    // Risk metrics
    println!("=== Risk Metrics ===");
    println!("Volatility (Ann): {:.2}%", results.volatility_ann * 100.0);
    println!("Sharpe Ratio: {:.2}", results.sharpe_ratio);
    println!("Sortino Ratio: {:.2}", results.sortino_ratio);
    println!("Calmar Ratio: {:.2}", results.calmar_ratio);
    println!("Max Drawdown: {:.2}%", results.max_drawdown * 100.0);
    println!("Avg Drawdown: {:.2}%", results.avg_drawdown * 100.0);
    println!(
        "Max DD Duration: {} days",
        results.max_drawdown_duration.num_days()
    );
    println!();

    // Trade statistics
    println!("=== Trade Statistics ===");
    println!("Number of Trades: {}", results.trades.len());
    if !results.trades.is_empty() {
        println!("Win Rate: {:.2}%", results.win_rate * 100.0);
        println!("Best Trade: ${:.2}", results.best_trade);
        println!("Worst Trade: ${:.2}", results.worst_trade);
        println!("Average Trade: ${:.2}", results.avg_trade);
        println!("Profit Factor: {:.2}", results.profit_factor);
        println!("Expectancy: ${:.2}", results.expectancy);
        println!("SQN: {:.2}", results.sqn);
        println!(
            "Avg Trade Duration: {} days",
            results.avg_trade_duration.num_days()
        );
    }

    Ok(())
}
