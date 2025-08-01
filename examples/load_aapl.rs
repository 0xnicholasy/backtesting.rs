use backtesting::data::DataLoader;
use backtesting::types::OHLCV;
use backtesting::{Backtest, BacktestConfig, Strategy};
use backtesting::{Order, OrderSide, OrderType};

struct SimpleStrategy;

impl Strategy for SimpleStrategy {
    fn init(&mut self, _data: &[OHLCV]) -> backtesting::Result<()> {
        println!("Initializing Simple Strategy");
        Ok(())
    }

    fn next(&mut self, bar: &OHLCV, index: usize) -> backtesting::Result<Vec<Order>> {
        // Buy on day 2, sell on day 6
        if index == 2 {
            println!("BUY at {:.2}", bar.close);
            Ok(vec![Order::new(
                OrderSide::Buy,
                OrderType::Market,
                60.0,
                None,
                None,
                None,
                None,
                None,
            )])
        } else if index == 6 {
            println!("SELL at {:.2}", bar.close);
            Ok(vec![Order::new(
                OrderSide::Sell,
                OrderType::Market,
                60.0,
                None,
                None,
                None,
                None,
                None,
            )])
        } else {
            Ok(vec![])
        }
    }
}

fn main() -> backtesting::Result<()> {
    println!("Loading AAPL Data Example");
    println!("========================");

    // Load AAPL data from CSV
    let data = DataLoader::load_from_csv("AAPL")?;

    println!("Loaded {} data points", data.len());
    println!(
        "Date range: {} to {}",
        data.first().unwrap().timestamp.format("%Y-%m-%d"),
        data.last().unwrap().timestamp.format("%Y-%m-%d")
    );

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
        commission: 0.001,
        ..Default::default()
    };

    // Run backtest
    let mut backtest = Backtest::new(&data, config);
    let results = backtest.run(SimpleStrategy)?;

    println!("\n=== Results ===");
    println!("Final Equity: ${:.2}", results.equity_final);
    println!("Total Return: {:.2}%", results.return_pct * 100.0);
    println!("Number of Trades: {}", results.trades.len());

    if let Some(trade) = results.trades.first() {
        println!("Trade P&L: ${:.2}", trade.pl());
        println!(
            "Entry: {:.2} -> Exit: {:.2}",
            trade.entry_price, 
            trade.exit_price.unwrap_or(0.0)
        );
    }

    Ok(())
}
