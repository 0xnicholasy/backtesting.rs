use backtesting::plotting::{BacktestPlotter, PlotConfig};
use backtesting::types::{Order, OrderSide, OrderType, OHLCV};
use backtesting::{Backtest, BacktestConfig, Strategy};
use chrono::{TimeZone, Utc};

struct SimpleTradingStrategy {
    position: bool,
    trade_count: usize,
}

impl SimpleTradingStrategy {
    fn new() -> Self {
        Self {
            position: false,
            trade_count: 0,
        }
    }
}

impl Strategy for SimpleTradingStrategy {
    fn init(&mut self, _data: &[OHLCV]) -> backtesting::Result<()> {
        println!("Initializing Simple Trading strategy");
        Ok(())
    }

    fn next(&mut self, bar: &OHLCV, index: usize) -> backtesting::Result<Vec<Order>> {
        // Simple strategy: buy every 50 bars, sell after 20 bars
        if !self.position && index % 50 == 10 && self.trade_count < 5 {
            self.position = true;
            println!("BUY at price: {:.2} (day {})", bar.close, index);

            Ok(vec![Order {
                side: OrderSide::Buy,
                order_type: OrderType::Market,
                size: 50.0,
                limit_price: None,
                stop_price: None,
                timestamp: bar.timestamp,
            }])
        } else if self.position && index % 20 == 0 {
            self.position = false;
            self.trade_count += 1;
            println!("SELL at price: {:.2} (day {})", bar.close, index);

            Ok(vec![Order {
                side: OrderSide::Sell,
                order_type: OrderType::Market,
                size: 50.0,
                limit_price: None,
                stop_price: None,
                timestamp: bar.timestamp,
            }])
        } else {
            Ok(vec![])
        }
    }
}

fn create_sample_data() -> Vec<OHLCV> {
    let mut data = Vec::new();
    let start_date = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();

    for i in 0..300 {
        // Create more volatile data with trends
        let base_trend = (i as f64 / 50.0).sin() * 15.0;
        let noise = (i as f64 * 0.7).sin() * 3.0 + (i as f64 * 0.3).cos() * 2.0;
        let price = 100.0 + base_trend + noise;
        let timestamp = start_date + chrono::Duration::days(i);

        data.push(OHLCV {
            timestamp,
            open: price - 0.5,
            high: price + 1.5,
            low: price - 1.5,
            close: price,
            volume: 1000000.0,
        });
    }

    data
}

fn main() -> backtesting::Result<()> {
    println!("Running Simple Trading backtest example");

    // Create sample data
    let data = create_sample_data();

    // Configure backtest
    let config = BacktestConfig {
        initial_cash: 10000.0,
        commission: 0.001, // 0.1% commission
        ..Default::default()
    };

    // Create and run backtest
    let mut backtest = Backtest::new(&data, config);
    let results = backtest.run(SimpleTradingStrategy::new())?;

    // Print results
    println!("\n=== Simple Trading Backtest Results ===");
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

    // Generate plots
    #[cfg(feature = "plotting")]
    {
        println!("\n=== Generating Plots ===");
        let plot_config = PlotConfig {
            width: 1200,
            height: 800,
            show_trades: true,
            show_equity_curve: true,
            show_drawdown: false,
        };

        match BacktestPlotter::plot(&data, &results, "simple_trading_backtest.png", plot_config) {
            Ok(_) => {
                println!("✓ Price chart with trades saved to: simple_trading_backtest.png");
                println!("✓ Equity curve saved to: simple_trading_backtest_equity.png");
            }
            Err(e) => println!("⚠ Failed to generate plots: {}", e),
        }
    }

    #[cfg(not(feature = "plotting"))]
    {
        println!("\n=== Plots ===");
        println!("Plotting is disabled. Run with --features plotting to enable plots.");
    }

    Ok(())
}
