use crate::types::{OHLCV, Order};
use crate::Result;

pub trait Strategy {
    /// Initialize the strategy with historical data
    /// This is called once before the backtest begins
    fn init(&mut self, data: &[OHLCV]) -> Result<()>;
    
    /// Called for each bar of data during the backtest
    /// Implement your trading logic here
    fn next(&mut self, bar: &OHLCV, index: usize) -> Result<Vec<Order>>;
    
    /// Optional: Called when a trade is closed
    fn on_trade_close(&mut self, _trade: &crate::types::Trade) -> Result<()> {
        Ok(())
    }
}

/// Base strategy struct that can be extended
pub struct BaseStrategy {
    pub name: String,
}

impl BaseStrategy {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
        }
    }
}

impl Strategy for BaseStrategy {
    fn init(&mut self, _data: &[OHLCV]) -> Result<()> {
        Ok(())
    }
    
    fn next(&mut self, _bar: &OHLCV, _index: usize) -> Result<Vec<Order>> {
        Ok(vec![])
    }
}