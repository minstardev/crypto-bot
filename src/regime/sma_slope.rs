use std::collections::VecDeque;

use super::{Regime, RegimeDetector};
use crate::model::Candle;
use crate::strategy::indicators::sma;

pub struct SmaSlopeDetector {
    name: String,
    period: usize,
    slope_window: usize,
    bull_threshold: f64,
    bear_threshold: f64,
    closes: VecDeque<f64>,
    sma_history: VecDeque<f64>,
}

impl SmaSlopeDetector {
    pub fn new(
        period: usize,
        slope_window: usize,
        bull_threshold_per_day: f64,
        bear_threshold_per_day: f64,
    ) -> Self {
        let name = format!(
            "SmaSlope(P{},W{},+{:.2}%/{:.2}%/d)",
            period,
            slope_window,
            bull_threshold_per_day * 100.0,
            bear_threshold_per_day * 100.0,
        );
        Self {
            name,
            period,
            slope_window,
            bull_threshold: bull_threshold_per_day,
            bear_threshold: bear_threshold_per_day,
            closes: VecDeque::with_capacity(period),
            sma_history: VecDeque::with_capacity(slope_window + 1),
        }
    }
}

impl RegimeDetector for SmaSlopeDetector {
    fn name(&self) -> &str {
        &self.name
    }

    fn detect(&mut self, candle: &Candle) -> Regime {
        if self.closes.len() == self.period {
            self.closes.pop_front();
        }
        self.closes.push_back(candle.close());
        if self.closes.len() < self.period {
            return Regime::Unknown;
        }

        let prices: Vec<f64> = self.closes.iter().copied().collect();
        let current_sma = sma(&prices);

        if self.sma_history.len() == self.slope_window + 1 {
            self.sma_history.pop_front();
        }
        self.sma_history.push_back(current_sma);

        if self.sma_history.len() < self.slope_window + 1 {
            return Regime::Unknown;
        }

        let past_sma = *self.sma_history.front().unwrap();
        if past_sma <= 0.0 {
            return Regime::Unknown;
        }
        let slope_per_day = ((current_sma - past_sma) / past_sma) / self.slope_window as f64;

        if slope_per_day >= self.bull_threshold {
            Regime::Bull
        } else if slope_per_day <= self.bear_threshold {
            Regime::Bear
        } else {
            Regime::Sideways
        }
    }

    fn reset(&mut self) {
        self.closes.clear();
        self.sma_history.clear();
    }
}
