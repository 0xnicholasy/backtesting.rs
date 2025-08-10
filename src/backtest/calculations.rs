use crate::trade::Trade;
use crate::types::OHLCV;
use chrono::{DateTime, Utc};

pub struct Calculations;

impl Calculations {
    pub fn calculate_exposure_time(data: &[OHLCV], trades: &[Trade]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let mut exposed_days = 0;
        let mut in_position = false;
        let mut position_start = data[0].timestamp;

        // Track position changes through trades
        for trade in trades {
            if !in_position {
                // Position opened
                in_position = true;
                position_start = trade.entry_time;
            } else {
                // Position closed
                if let Some(exit_time) = trade.exit_time {
                    exposed_days += (exit_time - position_start).num_days();
                }
                in_position = false;
            }
        }

        // If still in position at the end
        if in_position {
            exposed_days += (data.last().unwrap().timestamp - position_start).num_days();
        }

        let total_days =
            (data.last().unwrap().timestamp - data.first().unwrap().timestamp).num_days();
        if total_days > 0 {
            exposed_days as f64 / total_days as f64
        } else {
            0.0
        }
    }

    pub fn calculate_volatility(equity_curve: &[(DateTime<Utc>, f64)], years: f64) -> f64 {
        if equity_curve.len() < 2 || years <= 0.0 {
            return 0.0;
        }

        // Calculate daily returns
        let mut returns = Vec::with_capacity(equity_curve.len() - 1);
        for i in 1..equity_curve.len() {
            let prev_equity = equity_curve[i - 1].1;
            let curr_equity = equity_curve[i].1;
            if prev_equity > 0.0 {
                returns.push((curr_equity - prev_equity) / prev_equity);
            }
        }

        if returns.is_empty() {
            return 0.0;
        }

        // Calculate mean return
        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;

        // Calculate variance
        let variance = returns
            .iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>()
            / returns.len() as f64;

        // Annualized volatility (assuming daily data)
        variance.sqrt() * (252.0_f64).sqrt() // 252 trading days per year
    }

    pub fn calculate_drawdown_metrics(
        equity_curve: &[(DateTime<Utc>, f64)],
    ) -> (f64, f64, chrono::Duration, chrono::Duration) {
        if equity_curve.len() < 2 {
            return (0.0, 0.0, chrono::Duration::zero(), chrono::Duration::zero());
        }

        let mut max_drawdown = 0.0;
        let mut current_drawdown = 0.0;
        let mut peak_equity = equity_curve[0].1;
        let mut drawdown_start = equity_curve[0].0;
        let mut max_dd_duration = chrono::Duration::zero();
        let mut current_dd_duration = chrono::Duration::zero();
        let mut drawdowns = Vec::with_capacity(equity_curve.len() / 4);
        let mut dd_durations = Vec::with_capacity(equity_curve.len() / 4);

        for (timestamp, equity) in equity_curve {
            if *equity > peak_equity {
                // New peak, end of drawdown period
                if current_drawdown < 0.0 {
                    drawdowns.push(-current_drawdown);
                    dd_durations.push(current_dd_duration);
                }
                peak_equity = *equity;
                current_drawdown = 0.0;
                current_dd_duration = chrono::Duration::zero();
                drawdown_start = *timestamp;
            } else {
                // In drawdown
                current_drawdown = (*equity - peak_equity) / peak_equity;
                current_dd_duration = *timestamp - drawdown_start;

                if current_drawdown < max_drawdown {
                    max_drawdown = current_drawdown;
                    max_dd_duration = current_dd_duration;
                }
            }
        }

        // Handle final drawdown if still ongoing
        if current_drawdown < 0.0 {
            drawdowns.push(-current_drawdown);
            dd_durations.push(current_dd_duration);
        }

        let avg_drawdown = if drawdowns.is_empty() {
            0.0
        } else {
            -drawdowns.iter().sum::<f64>() / drawdowns.len() as f64
        };

        let avg_dd_duration = if dd_durations.is_empty() {
            chrono::Duration::zero()
        } else {
            let total_duration: chrono::Duration = dd_durations.iter().sum();
            total_duration / dd_durations.len() as i32
        };

        (max_drawdown, avg_drawdown, max_dd_duration, avg_dd_duration)
    }

    pub fn calculate_sortino_ratio(
        equity_curve: &[(DateTime<Utc>, f64)],
        return_ann: f64,
        risk_free_rate: f64,
    ) -> f64 {
        if equity_curve.len() < 2 {
            return 0.0;
        }

        // Calculate daily returns
        let mut returns = Vec::with_capacity(equity_curve.len() - 1);
        for i in 1..equity_curve.len() {
            let prev_equity = equity_curve[i - 1].1;
            let curr_equity = equity_curve[i].1;
            if prev_equity > 0.0 {
                returns.push((curr_equity - prev_equity) / prev_equity);
            }
        }

        if returns.is_empty() {
            return 0.0;
        }

        // Calculate downside deviation (only negative returns)
        let daily_risk_free = risk_free_rate / 252.0; // Daily risk-free rate
        let downside_returns: Vec<f64> = returns
            .iter()
            .filter_map(|&r| {
                if r < daily_risk_free {
                    Some(r - daily_risk_free)
                } else {
                    None
                }
            })
            .collect();

        if downside_returns.is_empty() {
            return if return_ann > risk_free_rate {
                f64::INFINITY
            } else {
                0.0
            };
        }

        let downside_variance =
            downside_returns.iter().map(|r| r.powi(2)).sum::<f64>() / downside_returns.len() as f64;

        let downside_deviation = downside_variance.sqrt() * (252.0_f64).sqrt(); // Annualized

        if downside_deviation > 0.0 {
            (return_ann - risk_free_rate) / downside_deviation
        } else {
            0.0
        }
    }

    pub fn calculate_sqn(trades: &[Trade]) -> f64 {
        if trades.len() < 2 {
            return 0.0;
        }

        let avg_trade = trades.iter().map(|t| t.pl()).sum::<f64>() / trades.len() as f64;

        // Calculate standard deviation of trade P&L
        let variance = trades
            .iter()
            .map(|t| (t.pl() - avg_trade).powi(2))
            .sum::<f64>()
            / trades.len() as f64;

        let std_dev = variance.sqrt();

        if std_dev > 0.0 {
            (trades.len() as f64).sqrt() * avg_trade / std_dev
        } else {
            0.0
        }
    }
}