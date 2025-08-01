use crate::order::Order;
use crate::trade::Trade;
use crate::types::OHLCV;
use crate::Result;

pub trait Strategy {
    /// Initialize the strategy with historical data
    /// This is called once before the backtest begins
    fn init(&mut self, data: &[OHLCV]) -> Result<()>;

    /// Called for each bar of data during the backtest
    /// Implement your trading logic here
    fn next(&mut self, bar: &OHLCV, index: usize) -> Result<Vec<Order>>;

    /// Optional: Called when a trade is closed
    fn on_trade_close(&mut self, _trade: &Trade) -> Result<()> {
        Ok(())
    }

    /// Place a new long order
    fn buy(
        &mut self,
        size: Option<f64>,
        limit: Option<f64>,
        stop: Option<f64>,
        sl: Option<f64>,
        tp: Option<f64>,
        tag: Option<String>,
    ) -> Order {
        use crate::order::{OrderSide, OrderType};

        Order::new(
            OrderSide::Buy,
            if limit.is_some() {
                OrderType::Limit
            } else {
                OrderType::Market
            },
            size.unwrap_or(0.9999),
            limit,
            stop,
            sl,
            tp,
            tag,
        )
    }

    /// Place a new short order
    fn sell(
        &mut self,
        size: Option<f64>,
        limit: Option<f64>,
        stop: Option<f64>,
        sl: Option<f64>,
        tp: Option<f64>,
        tag: Option<String>,
    ) -> Order {
        use crate::order::{OrderSide, OrderType};

        Order::new(
            OrderSide::Sell,
            if limit.is_some() {
                OrderType::Limit
            } else {
                OrderType::Market
            },
            size.unwrap_or(0.9999),
            limit,
            stop,
            sl,
            tp,
            tag,
        )
    }
}

/// Base strategy struct that can be extended
pub struct BaseStrategy {
    pub name: String,
}

impl BaseStrategy {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
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
