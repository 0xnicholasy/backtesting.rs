use crate::types::OHLCV;
use super::Indicator;

pub struct RSI {
    period: usize,
    gains: Vec<f64>,
    losses: Vec<f64>,
    avg_gain: f64,
    avg_loss: f64,
    prev_close: Option<f64>,
    is_initialized: bool,
}

impl RSI {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            gains: Vec::new(),
            losses: Vec::new(),
            avg_gain: 0.0,
            avg_loss: 0.0,
            prev_close: None,
            is_initialized: false,
        }
    }
}

impl Indicator for RSI {
    type Output = f64;
    
    fn update(&mut self, bar: &OHLCV) -> Option<f64> {
        let Some(prev_close) = self.prev_close else {
            self.prev_close = Some(bar.close);
            return None;
        };
        
        let change = bar.close - prev_close;
        let gain = if change > 0.0 { change } else { 0.0 };
        let loss = if change < 0.0 { -change } else { 0.0 };
        
        self.gains.push(gain);
        self.losses.push(loss);
        
        if self.gains.len() > self.period {
            self.gains.remove(0);
            self.losses.remove(0);
        }
        
        self.prev_close = Some(bar.close);
        
        if self.gains.len() == self.period {
            if !self.is_initialized {
                self.avg_gain = self.gains.iter().sum::<f64>() / self.period as f64;
                self.avg_loss = self.losses.iter().sum::<f64>() / self.period as f64;
                self.is_initialized = true;
            } else {
                self.avg_gain = (self.avg_gain * (self.period - 1) as f64 + gain) / self.period as f64;
                self.avg_loss = (self.avg_loss * (self.period - 1) as f64 + loss) / self.period as f64;
            }
            
            if self.avg_loss == 0.0 {
                return Some(100.0);
            }
            
            let rs = self.avg_gain / self.avg_loss;
            let rsi = 100.0 - (100.0 / (1.0 + rs));
            Some(rsi)
        } else {
            None
        }
    }
    
    fn current(&self) -> Option<f64> {
        if self.gains.len() == self.period && self.is_initialized {
            if self.avg_loss == 0.0 {
                return Some(100.0);
            }
            let rs = self.avg_gain / self.avg_loss;
            let rsi = 100.0 - (100.0 / (1.0 + rs));
            Some(rsi)
        } else {
            None
        }
    }
}