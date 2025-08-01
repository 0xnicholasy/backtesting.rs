use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::position::Position;
use crate::trade::Trade;
use crate::types::{DirectionalTrade, Executable, StopManagement};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
    PartiallyFilled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Order {
    pub side: OrderSide,
    pub order_type: OrderType,
    pub size: f64,
    pub limit: Option<f64>,
    pub stop: Option<f64>,
    pub sl: Option<f64>,
    pub tp: Option<f64>,
    pub tag: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub status: OrderStatus,
    pub filled_size: f64,
}

impl Order {
    pub fn new(
        side: OrderSide,
        order_type: OrderType,
        size: f64,
        limit: Option<f64>,
        stop: Option<f64>,
        sl: Option<f64>,
        tp: Option<f64>,
        tag: Option<String>,
    ) -> Self {
        Self {
            side,
            order_type,
            size,
            limit,
            stop,
            sl,
            tp,
            tag,
            timestamp: Utc::now(),
            status: OrderStatus::Pending,
            filled_size: 0.0,
        }
    }

    pub fn is_long(&self) -> bool {
        matches!(self.side, OrderSide::Buy) && self.size > 0.0
    }

    pub fn is_short(&self) -> bool {
        matches!(self.side, OrderSide::Sell) && self.size > 0.0
    }

    pub fn is_contingent(&self) -> bool {
        self.sl.is_some() || self.tp.is_some()
    }

    pub fn remaining_size(&self) -> f64 {
        self.size - self.filled_size
    }

    pub fn is_filled(&self) -> bool {
        matches!(self.status, OrderStatus::Filled)
    }

    pub fn is_cancelled(&self) -> bool {
        matches!(self.status, OrderStatus::Cancelled)
    }

    pub fn cancel(&mut self) {
        if matches!(self.status, OrderStatus::Pending | OrderStatus::PartiallyFilled) {
            self.status = OrderStatus::Cancelled;
        }
    }

    pub fn fill(&mut self, fill_size: f64, fill_price: f64, bar_index: usize, fill_time: DateTime<Utc>) -> Option<Trade> {
        if matches!(self.status, OrderStatus::Cancelled) {
            return None;
        }

        let actual_fill_size = fill_size.min(self.remaining_size());
        self.filled_size += actual_fill_size;

        if self.filled_size >= self.size {
            self.status = OrderStatus::Filled;
        } else {
            self.status = OrderStatus::PartiallyFilled;
        }

        Some(Trade::new(
            bar_index,
            fill_price,
            fill_time,
            if self.is_long() { actual_fill_size } else { -actual_fill_size },
            self.sl,
            self.tp,
            self.tag.clone(),
        ))
    }

    pub fn execute_to_position(&self, fill_price: f64, fill_time: DateTime<Utc>) -> Option<Position> {
        if !self.is_filled() {
            return None;
        }

        let position_size = if self.is_long() { self.size } else { -self.size };
        Some(Position::with_stops(position_size, fill_price, fill_time, self.sl, self.tp, self.tag.clone()))
    }
}

impl DirectionalTrade for Order {
    fn is_long(&self) -> bool {
        matches!(self.side, OrderSide::Buy) && self.size > 0.0
    }

    fn is_short(&self) -> bool {
        matches!(self.side, OrderSide::Sell) && self.size > 0.0
    }

    fn size(&self) -> f64 {
        self.size
    }
}

impl Executable for Order {
    type Position = Position;
    type Trade = Trade;

    fn execute(&mut self, fill_price: f64, fill_time: DateTime<Utc>, bar_index: usize) -> Option<Self::Trade> {
        self.fill(self.remaining_size(), fill_price, bar_index, fill_time)
    }

    fn to_position(&self, fill_price: f64, fill_time: DateTime<Utc>) -> Option<Self::Position> {
        self.execute_to_position(fill_price, fill_time)
    }
}

impl StopManagement for Order {
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