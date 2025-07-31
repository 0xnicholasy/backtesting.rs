use crate::types::OHLCV;
use super::Indicator;

pub struct OnBalanceVolume {
    prev_close: Option<f64>,
    obv_value: f64,
}

impl OnBalanceVolume {
    pub fn new() -> Self {
        Self {
            prev_close: None,
            obv_value: 0.0,
        }
    }
}

impl Default for OnBalanceVolume {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for OnBalanceVolume {
    type Output = f64;
    
    fn update(&mut self, bar: &OHLCV) -> Option<f64> {
        if let Some(prev_close) = self.prev_close {
            if bar.close > prev_close {
                self.obv_value += bar.volume;
            } else if bar.close < prev_close {
                self.obv_value -= bar.volume;
            }
        }
        
        self.prev_close = Some(bar.close);
        Some(self.obv_value)
    }
    
    fn current(&self) -> Option<f64> {
        if self.prev_close.is_some() {
            Some(self.obv_value)
        } else {
            None
        }
    }
}