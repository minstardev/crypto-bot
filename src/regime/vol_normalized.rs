use std::collections::VecDeque;

use super::{Regime, RegimeDetector};
use crate::model::Candle;

pub struct VolNormalizedDetector {
    name: String,
    period: usize,
    bull_threshold: f64,
    bear_threshold: f64,
    closes: VecDeque<f64>,
}

impl VolNormalizedDetector {
    pub fn new(period: usize, bull_threshold: f64, bear_threshold: f64) -> Self {
        let name = format!(
            "VolNorm(P{},+{:.2}/{:.2})",
            period, bull_threshold, bear_threshold
        );
        Self {
            name,
            period,
            bull_threshold,
            bear_threshold,
            closes: VecDeque::with_capacity(period + 1),
        }
    }
}

impl RegimeDetector for VolNormalizedDetector {
    fn name(&self) -> &str {
        &self.name
    }

    fn detect(&mut self, candle: &Candle) -> Regime {
        if self.closes.len() == self.period + 1 {
            self.closes.pop_front();
        }
        self.closes.push_back(candle.close());
        if self.closes.len() < self.period + 1 {
            return Regime::Unknown;
        }

        let prices: Vec<f64> = self.closes.iter().copied().collect();
        let returns: Vec<f64> = prices
            .windows(2)
            .map(|w| if w[0] > 0.0 { (w[1] - w[0]) / w[0] } else { 0.0 })
            .collect();

        let n = returns.len() as f64;
        if n < 2.0 {
            return Regime::Unknown;
        }
        let mean = returns.iter().sum::<f64>() / n;
        let var = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
        let std = var.sqrt();
        if std == 0.0 {
            return Regime::Unknown;
        }
        let score = mean / std * n.sqrt();

        if score >= self.bull_threshold {
            Regime::Bull
        } else if score <= self.bear_threshold {
            Regime::Bear
        } else {
            Regime::Sideways
        }
    }

    fn reset(&mut self) {
        self.closes.clear();
    }
}
