use backtesting::{Backtest, Strategy, BacktestConfig};
use backtesting::types::{OHLCV, Order, OrderType, OrderSide};
use chrono::{Utc, TimeZone};

struct BuyAndHold {
    bought: bool,
}

impl BuyAndHold {
    fn new() -> Self {
        Self { bought: false }
    }
}

impl Strategy for BuyAndHold {
    fn init(&mut self, _data: &[OHLCV]) -> backtesting::Result<()> {
        println!("Initializing Buy and Hold strategy");
        Ok(())
    }
    
    fn next(&mut self, bar: &OHLCV, _index: usize) -> backtesting::Result<Vec<Order>> {
        if !self.bought {
            self.bought = true;
            println!("Buying at price: {:.2}", bar.close);
            
            Ok(vec![Order {
                side: OrderSide::Buy,
                order_type: OrderType::Market,
                size: 99.0, // Reduced to account for commission
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
    // Create some sample data for demonstration
    let mut data = Vec::new();
    let start_date = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();
    
    for i in 0..252 { // One year of trading days
        let price = 100.0 + (i as f64 * 0.1) + (i as f64).sin() * 5.0;
        let timestamp = start_date + chrono::Duration::days(i);
        
        data.push(OHLCV {
            timestamp,
            open: price - 0.5,
            high: price + 1.0,
            low: price - 1.0,
            close: price,
            volume: 1000000.0,
        });
    }
    
    data
}

fn main() -> backtesting::Result<()> {
    println!("Running Buy and Hold backtest example");
    
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
    let results = backtest.run(BuyAndHold::new())?;
    
    // Print results
    println!("\n=== Backtest Results ===");
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
    println!("Buy & Hold Return: {:.2}%", results.buy_hold_return_pct * 100.0);
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
    println!("Max DD Duration: {} days", results.max_drawdown_duration.num_days());
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
        println!("Avg Trade Duration: {} days", results.avg_trade_duration.num_days());
    } else {
        println!("No completed trades (buy-and-hold strategy)");
    }
    
    Ok(())
}