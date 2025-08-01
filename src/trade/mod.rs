use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use crate::types::{DirectionalTrade, ProfitLoss, StopManagement};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trade {
    pub entry_bar: usize,
    pub entry_price: f64,
    pub entry_time: DateTime<Utc>,
    pub exit_bar: Option<usize>,
    pub exit_price: Option<f64>,
    pub exit_time: Option<DateTime<Utc>>,
    pub size: f64,
    pub sl: Option<f64>,
    pub tp: Option<f64>,
    pub tag: Option<String>,
}

impl Trade {
    pub fn new(
        entry_bar: usize,
        entry_price: f64,
        entry_time: DateTime<Utc>,
        size: f64,
        sl: Option<f64>,
        tp: Option<f64>,
        tag: Option<String>,
    ) -> Self {
        Self {
            entry_bar,
            entry_price,
            entry_time,
            exit_bar: None,
            exit_price: None,
            exit_time: None,
            size,
            sl,
            tp,
            tag,
        }
    }

    pub fn is_long(&self) -> bool {
        self.size > 0.0
    }

    pub fn is_short(&self) -> bool {
        self.size < 0.0
    }

    pub fn is_closed(&self) -> bool {
        self.exit_price.is_some()
    }

    pub fn pl(&self) -> f64 {
        match self.exit_price {
            Some(exit_price) => {
                if self.is_long() {
                    (exit_price - self.entry_price) * self.size
                } else {
                    (self.entry_price - exit_price) * self.size.abs()
                }
            }
            None => 0.0, // Active trade, no realized P&L yet
        }
    }

    pub fn pl_pct(&self) -> f64 {
        if self.entry_price == 0.0 {
            0.0
        } else {
            match self.exit_price {
                Some(exit_price) => {
                    if self.is_long() {
                        (exit_price - self.entry_price) / self.entry_price
                    } else {
                        (self.entry_price - exit_price) / self.entry_price
                    }
                }
                None => 0.0, // Active trade, no realized P&L yet
            }
        }
    }

    pub fn value(&self) -> f64 {
        match self.exit_price {
            Some(exit_price) => self.size * exit_price,
            None => self.size * self.entry_price, // Current value based on entry
        }
    }

    pub fn duration(&self) -> Duration {
        match self.exit_time {
            Some(exit_time) => exit_time - self.entry_time,
            None => Utc::now() - self.entry_time, // Duration so far for active trade
        }
    }

    pub fn close(&mut self, exit_bar: Option<usize>, exit_price: f64, exit_time: DateTime<Utc>) {
        self.exit_bar = exit_bar;
        self.exit_price = Some(exit_price);
        self.exit_time = Some(exit_time);
    }

    pub fn close_portion(&mut self, portion: f64) -> Self {
        let portion = portion.clamp(0.0, 1.0);
        let closed_size = self.size * portion;

        // Reduce current trade size
        self.size -= closed_size;

        // Create new trade for the closed portion
        Trade::new(
            self.entry_bar,
            self.entry_price,
            self.entry_time,
            closed_size,
            self.sl,
            self.tp,
            self.tag.clone(),
        )
    }

    pub fn get_exit_time(&self) -> DateTime<Utc> {
        self.exit_time.unwrap_or_else(|| Utc::now()) // Use exit time if available, otherwise current time
    }

    pub fn get_exit_price(&self, current_price: f64) -> f64 {
        self.exit_price.unwrap_or(current_price) // Use exit price if available, otherwise entry price
    }
}

impl DirectionalTrade for Trade {
    fn is_long(&self) -> bool {
        self.size > 0.0
    }

    fn is_short(&self) -> bool {
        self.size < 0.0
    }

    fn size(&self) -> f64 {
        self.size
    }
}

impl ProfitLoss for Trade {
    fn pl(&self) -> f64 {
        match self.exit_price {
            Some(exit_price) => {
                if self.is_long() {
                    (exit_price - self.entry_price) * self.size
                } else {
                    (self.entry_price - exit_price) * self.size.abs()
                }
            }
            None => 0.0, // Active trade, no realized P&L yet
        }
    }

    fn pl_pct(&self) -> f64 {
        if self.entry_price == 0.0 {
            0.0
        } else {
            match self.exit_price {
                Some(exit_price) => {
                    if self.is_long() {
                        (exit_price - self.entry_price) / self.entry_price
                    } else {
                        (self.entry_price - exit_price) / self.entry_price
                    }
                }
                None => 0.0, // Active trade, no realized P&L yet
            }
        }
    }
}

impl StopManagement for Trade {
    fn has_sl(&self) -> bool {
        self.sl.is_some()
    }

    fn has_tp(&self) -> bool {
        self.tp.is_some()
    }

    fn sl(&self) -> Option<f64> {
        self.sl
    }

    fn tp(&self) -> Option<f64> {
        self.tp
    }

    fn should_trigger_sl(&self, current_price: f64) -> bool {
        if let Some(sl) = self.sl {
            if self.is_long() {
                current_price <= sl
            } else {
                current_price >= sl
            }
        } else {
            false
        }
    }

    fn should_trigger_tp(&self, current_price: f64) -> bool {
        if let Some(tp) = self.tp {
            if self.is_long() {
                current_price >= tp
            } else {
                current_price <= tp
            }
        } else {
            false
        }
    }
}
