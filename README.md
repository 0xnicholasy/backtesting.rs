# Backtesting.rs

A Rust implementation of [backtesting.py](https://kernc.github.io/backtesting.py/) - a fast, flexible, and comprehensive framework for backtesting trading strategies.

## Features

- **High Performance**: Built in Rust for maximum speed and memory efficiency
- **Type Safety**: Leverages Rust's type system to prevent common trading logic errors
- **Flexible Strategy Framework**: Easy-to-implement Strategy trait for custom trading logic
- **Comprehensive Analytics**: Detailed performance metrics and risk analysis
- **Built-in Indicators**: Common technical indicators (SMA, EMA, RSI, etc.)
- **Optimization Support**: Parameter optimization with multiple algorithms
- **Plotting**: Optional visualization of backtest results
- **CSV Data Loading**: Easy loading of OHLCV data from CSV files
- **Flexible Data Formats**: Support for various CSV formats and date formats

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
backtesting = "0.1.0"
```

### Basic Example

```rust
use backtesting::{Backtest, Strategy, BacktestConfig};
use backtesting::types::{OHLCV, Order, OrderType, OrderSide};
use backtesting::data::DataLoader;

struct BuyAndHold {
    bought: bool,
}

impl Strategy for BuyAndHold {
    fn init(&mut self, _data: &[OHLCV]) -> backtesting::Result<()> {
        Ok(())
    }
    
    fn next(&mut self, bar: &OHLCV, _index: usize) -> backtesting::Result<Vec<Order>> {
        if !self.bought {
            self.bought = true;
            Ok(vec![Order {
                side: OrderSide::Buy,
                order_type: OrderType::Market,
                size: 99.0, // Account for commission
                limit_price: None,
                stop_price: None,
                timestamp: bar.timestamp,
            }])
        } else {
            Ok(vec![])
        }
    }
}

fn main() -> backtesting::Result<()> {
    // Load OHLCV data from CSV
    let data = DataLoader::load_from_csv("AAPL")?;
    
    // Configure backtest
    let config = BacktestConfig {
        initial_cash: 10000.0,
        commission: 0.001, // 0.1% commission
        ..Default::default()
    };
    
    // Create and run backtest
    let mut backtest = Backtest::new(data, config);
    let results = backtest.run(BuyAndHold { bought: false })?;
    
    println!("Total Return: {:.2}%", results.return_pct * 100.0);
    println!("Sharpe Ratio: {:.2}", results.sharpe_ratio);
    println!("Max Drawdown: {:.2}%", results.max_drawdown * 100.0);
    
    Ok(())
}
```

## Data Loading

The library includes a flexible CSV data loader that supports various formats:

### Loading from CSV

```rust
use backtesting::data::DataLoader;

// Load from data/AAPL.csv
let data = DataLoader::load_from_csv("AAPL")?;

// Load from custom path
let data = DataLoader::load_from_file("path/to/custom.csv")?;

// Create sample data for testing
DataLoader::create_sample_data()?;
let data = DataLoader::load_from_csv("sample")?;
```

### Supported CSV Formats

The data loader automatically handles various CSV formats:

```csv
# Standard format
Date,Open,High,Low,Close,Volume
2023-01-01,150.00,152.50,149.00,151.20,1000000

# Yahoo Finance format
Date,Open,High,Low,Close,Adj Close,Volume
2023-01-01,150.00,152.50,149.00,151.20,151.20,1000000

# Flexible column names (case-insensitive)
timestamp,open,high,low,close,volume
Datetime,Open,High,Low,Close,Volume
```

### Supported Date Formats

- ISO 8601: `2023-01-01T00:00:00Z`
- Date only: `2023-01-01`
- US format: `01/01/2023`
- European format: `01-01-2023`
- With time: `2023-01-01 00:00:00`

## Project Structure

```
backtesting.rs/
├── src/
│   ├── lib.rs              # Main library entry point
│   ├── backtest/           # Core backtesting engine
│   │   └── mod.rs          # Backtest struct and execution logic
│   ├── strategy/           # Strategy framework
│   │   └── mod.rs          # Strategy trait and base implementations
│   ├── types/              # Core data types
│   │   └── mod.rs          # OHLCV, Order, Trade, Position types
│   ├── indicators/         # Technical indicators
│   │   └── mod.rs          # SMA, EMA, RSI, MACD, etc.
│   ├── data/               # Data loading utilities
│   │   └── mod.rs          # CSV parser and data validation
│   ├── optimization/       # Parameter optimization (optional)
│   │   └── mod.rs          # Grid search, genetic algorithms
│   └── plotting/           # Visualization (optional)
│       └── mod.rs          # Chart generation and plotting
├── data/                   # CSV data files
│   ├── AAPL.csv           # Example stock data
│   └── sample.csv         # Generated sample data
├── examples/               # Example strategies
│   ├── buy_and_hold.rs
│   ├── sma_crossover.rs
│   ├── csv_data_loading.rs
│   └── load_aapl.rs
├── tests/                  # Integration tests
├── benches/                # Performance benchmarks
└── Cargo.toml
```

## Core Components

### Backtest Engine

The `Backtest` struct is the main engine that:
- Processes historical OHLCV data
- Executes strategy logic for each time step
- Manages positions, orders, and cash
- Calculates comprehensive performance metrics
- Handles commissions, slippage, and margin requirements

### Strategy Framework  

Implement the `Strategy` trait to define your trading logic:

```rust
pub trait Strategy {
    fn init(&mut self, data: &[OHLCV]) -> Result<()>;
    fn next(&mut self, bar: &OHLCV, index: usize) -> Result<Vec<Order>>;
    fn on_trade_close(&mut self, trade: &Trade) -> Result<()> { Ok(()) }
}
```

### Data Types

- **OHLCV**: Open, High, Low, Close, Volume data with timestamp
- **Order**: Buy/sell orders with various types (Market, Limit, Stop)
- **Position**: Current holdings with entry price and P&L tracking
- **Trade**: Completed trade with entry/exit details and performance metrics

### Technical Indicators

Built-in indicators following a common `Indicator` trait:
- Simple Moving Average (SMA)
- Exponential Moving Average (EMA)
- Relative Strength Index (RSI)
- MACD
- Bollinger Bands
- Stochastic Oscillator

## Features

### Core Features
- [x] Basic backtesting framework
- [x] Strategy trait system
- [x] OHLCV data handling
- [x] Order management (Market, Limit, Stop orders)
- [x] Position tracking
- [x] Commission handling
- [x] Basic performance metrics

### Advanced Features
- [ ] Complete performance analytics (Sharpe, Sortino, Calmar ratios)
- [ ] Drawdown analysis
- [ ] Risk metrics
- [ ] Parameter optimization
- [ ] Walk-forward analysis
- [ ] Monte Carlo simulation
- [ ] Multi-timeframe support
- [ ] Portfolio backtesting
- [ ] Slippage modeling
- [ ] Margin trading support

### Optional Features
- [ ] Plotting and visualization (`plotting` feature)
- [ ] Advanced optimization algorithms (`optimization` feature)
- [ ] Real-time data feeds
- [ ] Paper trading mode

## Performance

Backtesting.rs is designed for speed:
- Compiled Rust performance vs. Python interpretation overhead
- Efficient memory management with zero-copy operations where possible
- Parallel processing support for optimization
- Minimal allocations during backtest execution

## Comparison with backtesting.py

| Feature | backtesting.py | backtesting.rs |
|---------|---------------|----------------|
| Language | Python | Rust |
| Performance | Good | Excellent |
| Type Safety | Runtime | Compile-time |
| Memory Usage | Higher | Lower |
| Ecosystem | Mature | Growing |
| Learning Curve | Easy | Moderate |

## Examples

See the `examples/` directory for complete strategy implementations:

- **Buy and Hold**: Simple baseline strategy
- **SMA Crossover**: Moving average crossover system  
- **Mean Reversion**: RSI-based mean reversion strategy
- **Multi-timeframe**: Strategy using multiple timeframes
- **Portfolio**: Multi-asset portfolio backtesting

## Contributing

Contributions are welcome! Please see our [contributing guidelines](CONTRIBUTING.md) for details.

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

This project is inspired by and aims to be compatible with the excellent [backtesting.py](https://kernc.github.io/backtesting.py/) library by Kernc.