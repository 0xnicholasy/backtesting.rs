use crate::types::OHLCV;

pub trait Indicator {
    type Output;
    
    fn update(&mut self, bar: &OHLCV) -> Option<Self::Output>;
    fn current(&self) -> Option<Self::Output>;
}

mod sma;
mod ema;
mod rsi;
mod bollinger_bands;
mod obv;

pub use sma::SimpleMovingAverage;
pub use ema::ExponentialMovingAverage;
pub use rsi::RSI;
pub use bollinger_bands::{BollingerBands, BollingerBandsOutput};
pub use obv::OnBalanceVolume;