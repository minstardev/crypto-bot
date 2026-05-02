use std::collections::VecDeque;

use super::{Regime, RegimeDetector};
use crate::model::Candle;

pub struct LinRegSlopeDetector {
    name: String,
    period: usize,
    bull_threshold: f64,
    bear_threshold: f64,
    closes: VecDeque<f64>,
}

impl LinRegSlopeDetector {
    pub fn new(period: usize, bull_threshold_per_day: f64, bear_threshold_per_day: f64) -> Self {
        let name = format!(
            "LinReg(P{},+{:.2}%/{:.2}%/d)",
            period,
            bull_threshold_per_day * 100.0,
            bear_threshold_per_day * 100.0,
        );
        Self {
            name,
            period,
            bull_threshold: bull_threshold_per_day,
            bear_threshold: bear_threshold_per_day,
            closes: VecDeque::with_capacity(period),
        }
    }
}

impl RegimeDetector for LinRegSlopeDetector {
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

        let n = self.closes.len() as f64;
        let mean_x = (n - 1.0) / 2.0;
        let mean_y: f64 = self.closes.iter().sum::<f64>() / n;
        if mean_y <= 0.0 {
            return Regime::Unknown;
        }

        let mut num = 0.0f64;
        let mut den = 0.0f64;
        for (i, &y) in self.closes.iter().enumerate() {
            let x = i as f64;
            num += (x - mean_x) * (y - mean_y);
            den += (x - mean_x).powi(2);
        }
        if den == 0.0 {
            return Regime::Unknown;
        }
        let slope = num / den;
        let slope_per_day = slope / mean_y;

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
    }
}
