use backtesting::data::DataLoader;
use backtesting::{Order, OrderSide, OrderType};
use backtesting::types::OHLCV;
use backtesting::{Backtest, BacktestConfig, Strategy};

struct SimpleMAStrategy {
    ma_period: usize,
    prices: Vec<f64>,
    position: bool,
}

impl SimpleMAStrategy {
    fn new(ma_period: usize) -> Self {
        Self {
            ma_period,
            prices: Vec::new(),
            position: false,
        }
    }

    fn moving_average(&self) -> Option<f64> {
        if self.prices.len() < self.ma_period {
            None
        } else {
            let sum: f64 = self.prices.iter().rev().take(self.ma_period).sum();
            Some(sum / self.ma_period as f64)
        }
    }
}

impl Strategy for SimpleMAStrategy {
    fn init(&mut self, _data: &[OHLCV]) -> backtesting::Result<()> {
        println!(
            "Initializing Simple MA strategy with period {}",
            self.ma_period
        );
        Ok(())
    }

    fn next(&mut self, bar: &OHLCV, _index: usize) -> backtesting::Result<Vec<Order>> {
        self.prices.push(bar.close);

        if let Some(ma) = self.moving_average() {
            let mut orders = Vec::new();

            // Simple strategy: buy when price > MA, sell when price < MA
            if bar.close > ma * 1.02 && !self.position {
                self.position = true;
                // Calculate position size based on $10k capital - use ~95% of capital
                let target_value = 9500.0; // 95% of $10,000 initial capital
                let position_size = target_value / bar.close;
                orders.push(Order::new(
                    OrderSide::Buy,
                    OrderType::Market,
                    position_size,
                    None,
                    None,
                    None,
                    None,
                    None,
                ));
                println!("BUY at {:.2} (MA: {:.2}) - size: {:.2}", bar.close, ma, position_size);
            } else if bar.close < ma * 0.98 && self.position {
                self.position = false;
                // For sell orders, we'll sell all shares. The backtest engine will handle the actual position size
                orders.push(Order::new(
                    OrderSide::Sell,
                    OrderType::Market,
                    1000.0, // Large number to ensure we sell the full position
                    None,
                    None,
                    None,
                    None,
                    None,
                ));
                println!("SELL at {:.2} (MA: {:.2})", bar.close, ma);
            }

            Ok(orders)
        } else {
            Ok(vec![])
        }
    }
}

fn main() -> backtesting::Result<()> {
    println!("CSV Data Loading Example");
    println!("========================");

    // First, create sample data if it doesn't exist
    println!("Creating sample data...");
    DataLoader::create_sample_data()?;

    // Load data from CSV
    println!("Loading data from CSV...");
    let data = DataLoader::load_from_csv("S&P500")?;

    println!("Loaded {} data points", data.len());
    println!(
        "Date range: {} to {}",
        data.first().unwrap().timestamp.format("%Y-%m-%d"),
        data.last().unwrap().timestamp.format("%Y-%m-%d")
    );
    println!(
        "Price range: {:.2} to {:.2}",
        data.iter().map(|d| d.close).fold(f64::INFINITY, f64::min),
        data.iter()
            .map(|d| d.close)
            .fold(f64::NEG_INFINITY, f64::max)
    );

    // Configure backtest
    let config = BacktestConfig {
        initial_cash: 10000.0,
        commission: 0.001,
        ..Default::default()
    };

    // Create strategy
    let strategy = SimpleMAStrategy::new(20); // 20-day moving average

    // Run backtest
    println!("\nRunning backtest...");
    let mut backtest = Backtest::new(&data, config);
    let results = backtest.run(strategy)?;

    // Print results
    println!("\n=== CSV Data Backtest Results ===");
    println!("Start Date: {}", results.start_date.format("%Y-%m-%d"));
    println!("End Date: {}", results.end_date.format("%Y-%m-%d"));
    println!("Duration: {} days", results.duration.num_days());
    println!();

    // Performance metrics
    println!("=== Performance ===");
    println!("Initial Cash: ${:.2}", 10000.0);
    println!("Final Equity: ${:.2}", results.equity_final);
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
    println!("Max Drawdown: {:.2}%", results.max_drawdown * 100.0);
    println!();

    // Trade statistics
    println!("=== Trade Statistics ===");
    println!("Number of Trades: {}", results.trades.len());
    if !results.trades.is_empty() {
        println!("Win Rate: {:.2}%", results.win_rate * 100.0);
        println!("Average Trade: ${:.2}", results.avg_trade);
        println!("Profit Factor: {:.2}", results.profit_factor);
    }

    Ok(())
}
