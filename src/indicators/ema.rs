use crate::types::OHLCV;
use super::Indicator;

pub struct ExponentialMovingAverage {
    alpha: f64,
    current_value: Option<f64>,
}

impl ExponentialMovingAverage {
    pub fn new(window: usize) -> Self {
        let alpha = 2.0 / (window as f64 + 1.0);
        Self {
            alpha,
            current_value: None,
        }
    }
}

impl Indicator for ExponentialMovingAverage {
    type Output = f64;
    
    fn update(&mut self, bar: &OHLCV) -> Option<f64> {
        match self.current_value {
            None => {
                self.current_value = Some(bar.close);
                Some(bar.close)
            }
            Some(prev) => {
                let new_value = self.alpha * bar.close + (1.0 - self.alpha) * prev;
                self.current_value = Some(new_value);
                Some(new_value)
            }
        }
    }
    
    fn current(&self) -> Option<f64> {
        self.current_value
    }
}