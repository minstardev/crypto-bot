use std::collections::VecDeque;

use super::{Regime, RegimeDetector};
use crate::model::Candle;
use crate::strategy::indicators::sma;

pub struct PriceVsSmaDetector {
    name: String,
    period: usize,
    slope_window: usize,
    bull_above_pct: f64,
    bear_below_pct: f64,
    closes: VecDeque<f64>,
    sma_history: VecDeque<f64>,
}

impl PriceVsSmaDetector {
    pub fn new(
        period: usize,
        slope_window: usize,
        bull_above_pct: f64,
        bear_below_pct: f64,
    ) -> Self {
        let name = format!(
            "PriceVsSma(P{},W{},+{:.0}%/-{:.0}%)",
            period,
            slope_window,
            bull_above_pct * 100.0,
            bear_below_pct * 100.0,
        );
        Self {
            name,
            period,
            slope_window,
            bull_above_pct,
            bear_below_pct,
            closes: VecDeque::with_capacity(period),
            sma_history: VecDeque::with_capacity(slope_window + 1),
        }
    }
}

impl RegimeDetector for PriceVsSmaDetector {
    fn name(&self) -> &str {
        &self.name
    }

    fn detect(&mut self, candle: &Candle) -> Regime {
        let close = candle.close();
        if self.closes.len() == self.period {
            self.closes.pop_front();
        }
        self.closes.push_back(close);
        if self.closes.len() < self.period {
            return Regime::Unknown;
        }

        let prices: Vec<f64> = self.closes.iter().copied().collect();
        let current_sma = sma(&prices);

        if self.sma_history.len() == self.slope_window + 1 {
            self.sma_history.pop_front();
        }
        self.sma_history.push_back(current_sma);

        if self.sma_history.len() < self.slope_window + 1 || current_sma <= 0.0 {
            return Regime::Unknown;
        }

        let past_sma = *self.sma_history.front().unwrap();
        let sma_rising = current_sma > past_sma;
        let sma_falling = current_sma < past_sma;
        let above = (close - current_sma) / current_sma;

        if sma_rising && above >= self.bull_above_pct {
            Regime::Bull
        } else if sma_falling && above <= -self.bear_below_pct {
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
