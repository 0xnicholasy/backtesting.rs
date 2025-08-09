use crate::order::{Order, OrderSide};
use crate::position::Position;
use crate::strategy::Strategy;
use crate::trade::Trade;
use crate::types::OHLCV;
use crate::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

mod calculations;
use calculations::Calculations;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    pub initial_cash: f64,
    pub commission: f64,
    pub margin: f64,
    pub trade_on_open: bool,
    pub hedging: bool,
    pub exclusive_orders: bool,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_cash: 10000.0,
            commission: 0.0,
            margin: 1.0,
            trade_on_open: false,
            hedging: false,
            exclusive_orders: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResults {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub duration: chrono::Duration,
    pub exposure_time: f64,
    pub equity_final: f64,
    pub equity_peak: f64,
    pub return_pct: f64,
    pub buy_hold_return_pct: f64,
    pub return_ann: f64,
    pub volatility_ann: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,
    pub max_drawdown: f64,
    pub avg_drawdown: f64,
    pub max_drawdown_duration: chrono::Duration,
    pub avg_drawdown_duration: chrono::Duration,
    pub trades: Vec<Trade>,
    pub win_rate: f64,
    pub best_trade: f64,
    pub worst_trade: f64,
    pub avg_trade: f64,
    pub max_trade_duration: chrono::Duration,
    pub avg_trade_duration: chrono::Duration,
    pub profit_factor: f64,
    pub expectancy: f64,
    pub sqn: f64, // System Quality Number
}

pub struct Backtest<'a> {
    config: BacktestConfig,
    data: &'a [OHLCV],
    current_position: Option<Position>,
    cash: f64,
    equity_curve: Vec<(DateTime<Utc>, f64)>,
    trades: Vec<Trade>,
    orders: Vec<Order>,
    current_bar_index: usize,
    position_entry_bar: Option<usize>,
}

impl<'a> Backtest<'a> {
    pub fn new(data: &'a Vec<OHLCV>, config: BacktestConfig) -> Self {
        let cash = config.initial_cash;
        Self {
            config,
            data,
            current_position: None,
            cash,
            equity_curve: Vec::new(),
            trades: Vec::new(),
            orders: Vec::new(),
            current_bar_index: 0,
            position_entry_bar: None,
        }
    }

    pub fn run<S: Strategy>(&mut self, mut strategy: S) -> Result<BacktestResults> {
        // Initialize strategy
        strategy.init(self.data)?;

        // Run backtest
        for (index, bar) in self.data.iter().enumerate() {
            self.current_bar_index = index;

            // Get orders from strategy
            let orders = strategy.next(bar, index)?;

            // Process orders
            for order in orders {
                self.process_order(order, bar)?;
            }

            // Update position current price if we have one
            if let Some(ref mut position) = self.current_position {
                position.update_price(bar.close);
            }

            // Update equity curve
            let equity = self.calculate_equity(bar);
            self.equity_curve.push((bar.timestamp, equity));
        }

        // Calculate final results
        self.calculate_results()
    }

    fn process_order(&mut self, order: Order, bar: &OHLCV) -> Result<()> {
        let price = match self.config.trade_on_open {
            true => bar.open,
            false => bar.close,
        };

        match order.side {
            OrderSide::Buy => self.open_position(order.size, price, bar.timestamp, bar.close)?,
            OrderSide::Sell => self.close_position(order.size, price, bar)?,
        }

        Ok(())
    }

    fn open_position(
        &mut self,
        size: f64,
        price: f64,
        timestamp: DateTime<Utc>,
        current_price: f64,
    ) -> Result<()> {
        let cost = size * price * (1.0 + self.config.commission);

        if cost > self.cash {
            return Ok(()); // Insufficient funds
        }

        self.cash -= cost;

        if let Some(ref mut position) = self.current_position {
            // Update existing position
            let total_cost = position.size * position.entry_price + size * price;
            let total_size = position.size + size;
            position.entry_price = total_cost / total_size;
            position.size = total_size;
            position.update_price(current_price);
        } else {
            // Create new position
            let mut new_position = Position::new(size, price, timestamp);
            new_position.update_price(current_price);
            self.current_position = Some(new_position);
            self.position_entry_bar = Some(self.current_bar_index);
        }

        Ok(())
    }

    fn close_position(&mut self, size: f64, price: f64, current_bar: &OHLCV) -> Result<()> {
        if let Some(ref mut position) = self.current_position {
            let close_size = size.min(position.size);
            let proceeds = close_size * price * (1.0 - self.config.commission);
            let _cost_basis = close_size * position.entry_price;

            self.cash += proceeds;

            // Create trade record
            let mut trade = Trade::new(
                self.position_entry_bar.unwrap_or(0),
                position.entry_price,
                position.entry_time,
                close_size,
                None, // sl
                None, // tp
                None, // tag
            );
            trade.close(Some(self.current_bar_index), price, current_bar.timestamp);

            self.trades.push(trade);

            // Update position
            position.size -= close_size;
            if position.size <= 0.0 {
                self.current_position = None;
                self.position_entry_bar = None;
            }
        } else {
            // No position to close - this is expected behavior, just skip
        }

        Ok(())
    }

    fn calculate_equity(&self, _bar: &OHLCV) -> f64 {
        let mut equity = self.cash;

        if let Some(ref position) = self.current_position {
            equity += position.value();
        }

        equity
    }

    fn calculate_results(&mut self) -> Result<BacktestResults> {
        if self.data.is_empty() || self.equity_curve.is_empty() {
            return Err("No data or equity curve available".into());
        }

        let start_date = self.data.first().unwrap().timestamp;
        let end_date = self.data.last().unwrap().timestamp;
        let duration = end_date - start_date;

        let initial_equity = self.config.initial_cash;
        let final_equity = self.equity_curve.last().unwrap().1;

        let return_pct = (final_equity - initial_equity) / initial_equity;
        let buy_hold_return = (self.data.last().unwrap().close - self.data.first().unwrap().close)
            / self.data.first().unwrap().close;

        // Calculate basic trade statistics
        let winning_trades: Vec<_> = self.trades.iter().filter(|t| t.pl() > 0.0).collect();
        let losing_trades: Vec<_> = self.trades.iter().filter(|t| t.pl() < 0.0).collect();

        let win_rate = if self.trades.is_empty() {
            0.0
        } else {
            winning_trades.len() as f64 / self.trades.len() as f64
        };

        let best_trade = self.trades.iter().map(|t| t.pl()).fold(0.0, f64::max);
        let worst_trade = self.trades.iter().map(|t| t.pl()).fold(0.0, f64::min);
        let avg_trade = if self.trades.is_empty() {
            0.0
        } else {
            self.trades.iter().map(|t| t.pl()).sum::<f64>() / self.trades.len() as f64
        };

        // Calculate exposure time
        let exposure_time = Calculations::calculate_exposure_time(self.data, &self.trades);

        // Calculate annualized metrics
        let years = duration.num_days() as f64 / 365.25;
        let return_ann = if years > 0.0 {
            (1.0 + return_pct).powf(1.0 / years) - 1.0
        } else {
            0.0
        };

        // Calculate volatility (annualized standard deviation of returns)
        let volatility_ann = Calculations::calculate_volatility(&self.equity_curve, years);

        // Calculate drawdown metrics
        let (max_drawdown, avg_drawdown, max_dd_duration, avg_dd_duration) =
            Calculations::calculate_drawdown_metrics(&self.equity_curve);

        // Calculate risk-adjusted ratios
        let risk_free_rate = 0.02; // Assume 2% risk-free rate
        let sharpe_ratio = if volatility_ann > 0.0 {
            (return_ann - risk_free_rate) / volatility_ann
        } else {
            0.0
        };

        let sortino_ratio =
            Calculations::calculate_sortino_ratio(&self.equity_curve, return_ann, risk_free_rate);

        let calmar_ratio = if max_drawdown.abs() > 0.0 {
            return_ann / max_drawdown.abs()
        } else {
            0.0
        };

        // Calculate profit factor
        let gross_profit: f64 = winning_trades.iter().map(|t| t.pl()).sum();
        let gross_loss: f64 = losing_trades.iter().map(|t| t.pl().abs()).sum();
        let profit_factor = if gross_loss > 0.0 {
            gross_profit / gross_loss
        } else if gross_profit > 0.0 {
            f64::INFINITY
        } else {
            0.0
        };

        // Calculate System Quality Number (SQN)
        let sqn = Calculations::calculate_sqn(&self.trades);

        Ok(BacktestResults {
            start_date,
            end_date,
            duration,
            exposure_time,
            equity_final: final_equity,
            equity_peak: self
                .equity_curve
                .iter()
                .map(|(_, e)| *e)
                .fold(0.0, f64::max),
            return_pct,
            buy_hold_return_pct: buy_hold_return,
            return_ann,
            volatility_ann,
            sharpe_ratio,
            sortino_ratio,
            calmar_ratio,
            max_drawdown,
            avg_drawdown,
            max_drawdown_duration: max_dd_duration,
            avg_drawdown_duration: avg_dd_duration,
            trades: std::mem::take(&mut self.trades),
            win_rate,
            best_trade,
            worst_trade,
            avg_trade,
            max_trade_duration: self
                .trades
                .iter()
                .map(|t| t.duration())
                .max()
                .unwrap_or(chrono::Duration::zero()),
            avg_trade_duration: if self.trades.is_empty() {
                chrono::Duration::zero()
            } else {
                let total_duration: chrono::Duration =
                    self.trades.iter().map(|t| t.duration()).sum();
                total_duration / self.trades.len() as i32
            },
            profit_factor,
            expectancy: avg_trade,
            sqn,
        })
    }
}
