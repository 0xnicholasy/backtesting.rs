//! # Backtesting.rs
//!
//! A Rust implementation of backtesting.py for backtesting trading strategies.
//!
//! This library provides a framework for testing trading strategies against historical data,
//! with support for optimization, plotting, and comprehensive performance analysis.

pub mod backtest;
pub mod strategy;
pub mod types;
pub mod indicators;
pub mod data;

#[cfg(feature = "optimization")]
pub mod optimization;

#[cfg(feature = "plotting")]
pub mod plotting;

// Re-export main types for convenience
pub use backtest::{Backtest, BacktestConfig, BacktestResults};
pub use strategy::Strategy;
pub use types::{Order, Position, Trade, OrderType, OrderSide};

/// Result type used throughout the library
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;