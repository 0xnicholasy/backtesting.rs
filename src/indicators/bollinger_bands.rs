use crate::types::OHLCV;
use super::Indicator;

#[derive(Debug, Clone)]
pub struct BollingerBandsOutput {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
}

pub struct BollingerBands {
    period: usize,
    std_dev: f64,
    values: Vec<f64>,
}

impl BollingerBands {
    pub fn new(period: usize, std_dev: f64) -> Self {
        Self {
            period,
            std_dev,
            values: Vec::with_capacity(period),
        }
    }
    
    fn calculate_std_dev(&self, mean: f64) -> f64 {
        if self.values.len() < 2 {
            return 0.0;
        }
        
        let variance = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / self.values.len() as f64;
        
        variance.sqrt()
    }
}

impl Indicator for BollingerBands {
    type Output = BollingerBandsOutput;
    
    fn update(&mut self, bar: &OHLCV) -> Option<BollingerBandsOutput> {
        self.values.push(bar.close);
        
        if self.values.len() > self.period {
            self.values.remove(0);
        }
        
        if self.values.len() == self.period {
            let mean = self.values.iter().sum::<f64>() / self.period as f64;
            let std_deviation = self.calculate_std_dev(mean);
            
            Some(BollingerBandsOutput {
                upper: mean + (self.std_dev * std_deviation),
                middle: mean,
                lower: mean - (self.std_dev * std_deviation),
            })
        } else {
            None
        }
    }
    
    fn current(&self) -> Option<BollingerBandsOutput> {
        if self.values.len() == self.period {
            let mean = self.values.iter().sum::<f64>() / self.period as f64;
            let std_deviation = self.calculate_std_dev(mean);
            
            Some(BollingerBandsOutput {
                upper: mean + (self.std_dev * std_deviation),
                middle: mean,
                lower: mean - (self.std_dev * std_deviation),
            })
        } else {
            None
        }
    }
}