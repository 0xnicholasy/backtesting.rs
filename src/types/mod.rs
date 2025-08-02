use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OHLCV {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl OHLCV {
    pub fn new(
        timestamp: DateTime<Utc>,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Self {
        Self {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
        }
    }
}

impl Default for OHLCV {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            open: 0.0,
            high: 0.0,
            low: 0.0,
            close: 0.0,
            volume: 0.0,
        }
    }
}

/// Common trait for trading instruments that have position direction
pub trait DirectionalTrade {
    fn is_long(&self) -> bool;
    fn is_short(&self) -> bool;
    fn size(&self) -> f64;
}

/// Trait for calculating profit and loss
pub trait ProfitLoss {
    fn pl(&self) -> f64;
    fn pl_pct(&self) -> f64;
}

/// Trait for items that can be closed or exited
pub trait Closeable {
    type Output;

    fn close(&mut self, exit_price: f64, exit_time: DateTime<Utc>) -> Self::Output;
}

/// Trait for orders that can be executed
pub trait Executable {
    type Position;
    type Trade;

    fn execute(
        &mut self,
        fill_price: f64,
        fill_time: DateTime<Utc>,
        bar_index: usize,
    ) -> Option<Self::Trade>;
    fn to_position(&self, fill_price: f64, fill_time: DateTime<Utc>) -> Option<Self::Position>;
}

/// Trait for managing stop loss and take profit levels
pub trait StopManagement {
    fn has_sl(&self) -> bool;
    fn has_tp(&self) -> bool;
    fn sl(&self) -> Option<f64>;
    fn tp(&self) -> Option<f64>;
    fn should_trigger_sl(&self, current_price: f64) -> bool;
    fn should_trigger_tp(&self, current_price: f64) -> bool;
}
