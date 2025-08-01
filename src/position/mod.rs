use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::trade::Trade;
use crate::types::{DirectionalTrade, ProfitLoss, Closeable, StopManagement};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub size: f64,
    pub entry_price: f64,
    pub entry_time: DateTime<Utc>,
    pub current_price: f64,
    pub sl: Option<f64>,
    pub tp: Option<f64>,
    pub tag: Option<String>,
}

impl Position {
    pub fn new(size: f64, entry_price: f64, entry_time: DateTime<Utc>) -> Self {
        Self {
            size,
            entry_price,
            entry_time,
            current_price: entry_price,
            sl: None,
            tp: None,
            tag: None,
        }
    }

    pub fn with_stops(
        size: f64,
        entry_price: f64,
        entry_time: DateTime<Utc>,
        sl: Option<f64>,
        tp: Option<f64>,
        tag: Option<String>,
    ) -> Self {
        Self {
            size,
            entry_price,
            entry_time,
            current_price: entry_price,
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

    pub fn pl(&self) -> f64 {
        if self.is_long() {
            (self.current_price - self.entry_price) * self.size
        } else {
            (self.entry_price - self.current_price) * self.size.abs()
        }
    }

    pub fn pl_pct(&self) -> f64 {
        if self.entry_price == 0.0 {
            0.0
        } else if self.is_long() {
            (self.current_price - self.entry_price) / self.entry_price
        } else {
            (self.entry_price - self.current_price) / self.entry_price
        }
    }

    pub fn value(&self) -> f64 {
        self.size * self.current_price
    }

    pub fn update_price(&mut self, price: f64) {
        self.current_price = price;
    }

    pub fn close(&mut self, portion: f64, exit_price: f64, exit_time: DateTime<Utc>, exit_bar: Option<usize>) -> Trade {
        let portion = portion.clamp(0.0, 1.0);
        let closed_size = self.size * portion;
        self.size -= closed_size;

        let mut trade = Trade::new(
            exit_bar.unwrap_or(0),
            self.entry_price,
            self.entry_time,
            closed_size,
            self.sl,
            self.tp,
            self.tag.clone(),
        );
        trade.close(exit_bar, exit_price, exit_time);
        trade
    }

    pub fn close_all(&mut self, exit_price: f64, exit_time: DateTime<Utc>, exit_bar: Option<usize>) -> Trade {
        self.close(1.0, exit_price, exit_time, exit_bar)
    }

    pub fn should_trigger_sl(&self) -> bool {
        if let Some(sl) = self.sl {
            if self.is_long() {
                self.current_price <= sl
            } else {
                self.current_price >= sl
            }
        } else {
            false
        }
    }

    pub fn should_trigger_tp(&self) -> bool {
        if let Some(tp) = self.tp {
            if self.is_long() {
                self.current_price >= tp
            } else {
                self.current_price <= tp
            }
        } else {
            false
        }
    }
}

impl DirectionalTrade for Position {
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

impl ProfitLoss for Position {
    fn pl(&self) -> f64 {
        if self.is_long() {
            (self.current_price - self.entry_price) * self.size
        } else {
            (self.entry_price - self.current_price) * self.size.abs()
        }
    }

    fn pl_pct(&self) -> f64 {
        if self.entry_price == 0.0 {
            0.0
        } else if self.is_long() {
            (self.current_price - self.entry_price) / self.entry_price
        } else {
            (self.entry_price - self.current_price) / self.entry_price
        }
    }
}

impl Closeable for Position {
    type Output = Trade;
    
    fn close(&mut self, exit_price: f64, exit_time: DateTime<Utc>) -> Self::Output {
        self.close_all(exit_price, exit_time, None)
    }
}

impl StopManagement for Position {
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

impl From<Position> for bool {
    fn from(position: Position) -> bool {
        position.size != 0.0
    }
}

impl From<&Position> for bool {
    fn from(position: &Position) -> bool {
        position.size != 0.0
    }
}