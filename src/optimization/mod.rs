use crate::backtest::{Backtest, BacktestConfig, BacktestResults};
use crate::strategy::Strategy;
use crate::types::OHLCV;
use rayon::prelude::*;
use std::collections::HashMap;

pub trait OptimizationMetric {
    fn calculate(&self, results: &BacktestResults) -> f64;
}

pub struct SharpeRatio;
impl OptimizationMetric for SharpeRatio {
    fn calculate(&self, results: &BacktestResults) -> f64 {
        results.sharpe_ratio
    }
}

pub struct TotalReturn;
impl OptimizationMetric for TotalReturn {
    fn calculate(&self, results: &BacktestResults) -> f64 {
        results.return_pct
    }
}

pub struct ProfitFactor;
impl OptimizationMetric for ProfitFactor {
    fn calculate(&self, results: &BacktestResults) -> f64 {
        results.profit_factor
    }
}

#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub parameters: HashMap<String, f64>,
    pub metric_value: f64,
    pub results: BacktestResults,
}

pub struct GridSearchOptimizer {
    pub max_workers: Option<usize>,
}

impl GridSearchOptimizer {
    pub fn new() -> Self {
        Self { max_workers: None }
    }

    pub fn with_max_workers(mut self, workers: usize) -> Self {
        self.max_workers = Some(workers);
        self
    }

    pub fn optimize<S, F, M>(
        &self,
        data: &Vec<OHLCV>,
        config: &BacktestConfig,
        strategy_factory: F,
        parameter_ranges: HashMap<String, Vec<f64>>,
        metric: M,
    ) -> crate::Result<OptimizationResult>
    where
        S: Strategy + Send,
        F: Fn(&HashMap<String, f64>) -> S + Send + Sync,
        M: OptimizationMetric + Send + Sync,
    {
        // Generate all parameter combinations
        let combinations = self.generate_combinations(&parameter_ranges);

        // Run backtests in parallel
        let results: Vec<_> = combinations
            .into_par_iter()
            .map(|params| {
                let strategy = strategy_factory(&params);
                let mut backtest = Backtest::new(data, config.clone());

                match backtest.run(strategy) {
                    Ok(results) => {
                        let metric_value = metric.calculate(&results);
                        Some(OptimizationResult {
                            parameters: params,
                            metric_value,
                            results,
                        })
                    }
                    Err(_) => None,
                }
            })
            .filter_map(|x| x)
            .collect();

        // Find best result
        results
            .into_iter()
            .max_by(|a, b| a.metric_value.partial_cmp(&b.metric_value).unwrap())
            .ok_or_else(|| "No valid optimization results found".into())
    }

    fn generate_combinations(
        &self,
        ranges: &HashMap<String, Vec<f64>>,
    ) -> Vec<HashMap<String, f64>> {
        let mut combinations = vec![HashMap::new()];

        for (param_name, values) in ranges {
            let mut new_combinations = Vec::new();

            for combination in &combinations {
                for &value in values {
                    let mut new_combination = combination.clone();
                    new_combination.insert(param_name.clone(), value);
                    new_combinations.push(new_combination);
                }
            }

            combinations = new_combinations;
        }

        combinations
    }
}

impl Default for GridSearchOptimizer {
    fn default() -> Self {
        Self::new()
    }
}
