use crate::backtest::BacktestResults;
use crate::types::OHLCV;
use crate::Trade;
use chrono::{DateTime, Utc};
use plotters::coord::types::RangedCoordf64;
use plotters::prelude::*;

pub struct PlotConfig {
    pub width: u32,
    pub height: u32,
    pub show_trades: bool,
    pub show_equity_curve: bool,
    pub show_drawdown: bool,
}

impl Default for PlotConfig {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 768,
            show_trades: true,
            show_equity_curve: true,
            show_drawdown: false,
        }
    }
}

pub struct BacktestPlotter;

impl BacktestPlotter {
    pub fn plot(
        data: &[OHLCV],
        results: &BacktestResults,
        output_path: &str,
        config: PlotConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let latest_price = data.last().unwrap_or(&OHLCV::default()).close;
        let root =
            BitMapBackend::new(output_path, (config.width, config.height)).into_drawing_area();
        root.fill(&WHITE)?;

        let price_range = Self::find_price_range(data);
        let mut chart = ChartBuilder::on(&root)
            .caption("Backtest Results", ("sans-serif", 40))
            .margin(5)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(
                data.first().unwrap().timestamp..data.last().unwrap().timestamp,
                price_range,
            )?;

        chart.configure_mesh().draw()?;

        // Plot price data
        chart
            .draw_series(LineSeries::new(
                data.iter().map(|bar| (bar.timestamp, bar.close)),
                &BLACK,
            ))?
            .label("Price")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &BLACK));

        // Plot trades if enabled
        if config.show_trades {
            Self::plot_trades(&mut chart, &results.trades, latest_price)?;
        }

        chart.configure_series_labels().draw()?;
        root.present()?;

        // Plot equity curve if enabled
        if config.show_equity_curve {
            Self::plot_equity_curve(
                results,
                output_path.replace(".png", "_equity.png").as_str(),
                &config,
            )?;
        }

        Ok(())
    }

    fn find_price_range(data: &[OHLCV]) -> RangedCoordf64 {
        let min_price = data.iter().map(|bar| bar.low).fold(f64::INFINITY, f64::min);
        let max_price = data
            .iter()
            .map(|bar| bar.high)
            .fold(f64::NEG_INFINITY, f64::max);
        let padding = (max_price - min_price) * 0.1;
        ((min_price - padding)..(max_price + padding)).into()
    }

    fn plot_trades<DB: DrawingBackend>(
        chart: &mut ChartContext<
            '_,
            DB,
            Cartesian2d<RangedDateTime<DateTime<Utc>>, RangedCoordf64>,
        >,
        trades: &[Trade],
        last_price: f64,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        DB::ErrorType: 'static,
    {
        // Plot buy points (green)
        chart
            .draw_series(trades.iter().map(|trade| {
                Circle::new((trade.entry_time, trade.entry_price), 3, GREEN.filled())
            }))?
            .label("Buy")
            .legend(|(x, y)| Circle::new((x + 5, y), 3, GREEN.filled()));

        // Plot sell points (red)
        chart
            .draw_series(trades.iter().map(|trade| {
                Circle::new(
                    (trade.get_exit_time(), trade.get_exit_price(last_price)),
                    3,
                    RED.filled(),
                )
            }))?
            .label("Sell")
            .legend(|(x, y)| Circle::new((x + 5, y), 3, RED.filled()));

        Ok(())
    }

    fn plot_equity_curve(
        results: &BacktestResults,
        output_path: &str,
        config: &PlotConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root =
            BitMapBackend::new(output_path, (config.width, config.height)).into_drawing_area();
        root.fill(&WHITE)?;

        // Calculate equity curve from trades
        let mut equity_points = vec![(results.start_date, 10000.0)]; // Initial cash
        let mut current_equity = 10000.0;

        for trade in &results.trades {
            current_equity += trade.pl();
            equity_points.push((trade.get_exit_time(), current_equity));
        }

        let min_equity = equity_points
            .iter()
            .map(|(_, e)| *e)
            .fold(f64::INFINITY, f64::min);
        let max_equity = equity_points
            .iter()
            .map(|(_, e)| *e)
            .fold(f64::NEG_INFINITY, f64::max);
        let padding = (max_equity - min_equity) * 0.1;

        let equity_range: RangedCoordf64 = if min_equity == max_equity {
            ((min_equity - 1000.0)..(max_equity + 1000.0)).into()
        } else {
            ((min_equity - padding)..(max_equity + padding)).into()
        };

        let mut chart = ChartBuilder::on(&root)
            .caption("Equity Curve", ("sans-serif", 40))
            .margin(5)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(results.start_date..results.end_date, equity_range)?;

        chart.configure_mesh().draw()?;

        chart
            .draw_series(LineSeries::new(equity_points, &BLUE))?
            .label("Equity")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &BLUE));

        chart.configure_series_labels().draw()?;
        root.present()?;

        Ok(())
    }
}
