use backtesting::{Order, OrderSide, OrderType};
use backtesting::types::OHLCV;
use backtesting::{Backtest, BacktestConfig, Strategy};
use chrono::{TimeZone, Utc};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

struct SimpleBuyAndHold {
    bought: bool,
}

impl SimpleBuyAndHold {
    fn new() -> Self {
        Self { bought: false }
    }
}

impl Strategy for SimpleBuyAndHold {
    fn init(&mut self, _data: &[OHLCV]) -> backtesting::Result<()> {
        Ok(())
    }

    fn next(&mut self, bar: &OHLCV, _index: usize) -> backtesting::Result<Vec<Order>> {
        if !self.bought {
            self.bought = true;
            Ok(vec![Order::new(
                OrderSide::Buy,
                OrderType::Market,
                100.0,
                None,
                None,
                None,
                None,
                None,
            )])
        } else {
            Ok(vec![])
        }
    }
}

fn create_benchmark_data(size: usize) -> Vec<OHLCV> {
    let mut data = Vec::with_capacity(size);
    let start_date = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();

    for i in 0..size {
        let price = 100.0 + (i as f64 * 0.01);
        let timestamp = start_date + chrono::Duration::days(i as i64);

        data.push(OHLCV {
            timestamp,
            open: price - 0.1,
            high: price + 0.2,
            low: price - 0.2,
            close: price,
            volume: 10000.0,
        });
    }

    data
}

fn benchmark_backtest_small(c: &mut Criterion) {
    let data = create_benchmark_data(252); // 1 year
    let config = BacktestConfig::default();

    c.bench_function("backtest_small_252_days", |b| {
        b.iter(|| {
            let data_clone = data.clone();
            let mut backtest = Backtest::new(black_box(&data_clone), black_box(config.clone()));
            let strategy = SimpleBuyAndHold::new();
            black_box(backtest.run(strategy).unwrap())
        })
    });
}

fn benchmark_backtest_medium(c: &mut Criterion) {
    let data = create_benchmark_data(1260); // 5 years
    let config = BacktestConfig::default();

    c.bench_function("backtest_medium_1260_days", |b| {
        b.iter(|| {
            let data_clone = data.clone();
            let mut backtest = Backtest::new(black_box(&data_clone), black_box(config.clone()));
            let strategy = SimpleBuyAndHold::new();
            black_box(backtest.run(strategy).unwrap())
        })
    });
}

fn benchmark_backtest_large(c: &mut Criterion) {
    let data_size = 365 * 50; // data for 50 years
    let data = create_benchmark_data(data_size); // 50 years
    let config = BacktestConfig::default();

    c.bench_function("backtest_large_2520_days", |b| {
        b.iter(|| {
            let data_clone = data.clone();
            let mut backtest = Backtest::new(black_box(&data_clone), black_box(config.clone()));
            let strategy = SimpleBuyAndHold::new();
            black_box(backtest.run(strategy).unwrap())
        })
    });
}

criterion_group!(
    benches,
    benchmark_backtest_small,
    benchmark_backtest_medium,
    benchmark_backtest_large
);
criterion_main!(benches);
