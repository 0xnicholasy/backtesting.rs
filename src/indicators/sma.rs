use crate::types::OHLCV;
use super::Indicator;
use std::collections::VecDeque;

pub struct SimpleMovingAverage {
    window: usize,
    values: VecDeque<f64>,
    sum: f64,
}

impl SimpleMovingAverage {
    pub fn new(window: usize) -> Self {
        Self {
            window,
            values: VecDeque::with_capacity(window),
            sum: 0.0,
        }
    }
}

impl Indicator for SimpleMovingAverage {
    type Output = f64;
    
    fn update(&mut self, bar: &OHLCV) -> Option<f64> {
        self.values.push_back(bar.close);
        self.sum += bar.close;
        
        if self.values.len() > self.window {
            let old_value = self.values.pop_front().unwrap();
            self.sum -= old_value;
        }
        
        if self.values.len() == self.window {
            Some(self.sum / self.window as f64)
        } else {
            None
        }
    }
    
    fn current(&self) -> Option<f64> {
        if self.values.len() == self.window {
            Some(self.sum / self.window as f64)
        } else {
            None
        }
    }
}