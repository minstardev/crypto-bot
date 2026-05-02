use std::collections::VecDeque;

use super::indicators;
use super::{MarketState, Signal, Strategy};
use crate::model::Candle;

pub struct RsiReversion {
    name: String,
    period: usize,
    oversold: f64,
    target: f64,
    stop_loss_pct: f64,
    closes: VecDeque<f64>,
}

impl RsiReversion {
    pub fn new(period: usize, oversold: f64, target: f64, stop_loss_pct: f64) -> Self {
        let name = format!(
            "RSI(P{},os{:.0},t{:.0},sl{:.0}%)",
            period,
            oversold,
            target,
            stop_loss_pct * 100.0,
        );
        Self {
            name,
            period,
            oversold,
            target,
            stop_loss_pct,
            closes: VecDeque::with_capacity(period + 1),
        }
    }

    pub fn default_14() -> Self {
        Self::new(14, 30.0, 50.0, 0.05)
    }
}

impl Strategy for RsiReversion {
    fn name(&self) -> &str {
        &self.name
    }

    fn on_candle(&mut self, candle: &Candle, state: &MarketState) -> Signal {
        let close = candle.close();
        if self.closes.len() == self.period + 1 {
            self.closes.pop_front();
        }
        self.closes.push_back(close);

        if self.closes.len() < self.period + 1 {
            return Signal::Hold;
        }

        let prices: Vec<f64> = self.closes.iter().copied().collect();
        let rsi = indicators::rsi(&prices, self.period);

        if state.position > 0.0 && state.avg_buy_price > 0.0 {
            let pnl_pct = (close - state.avg_buy_price) / state.avg_buy_price;
            if pnl_pct < -self.stop_loss_pct {
                return Signal::Sell { ratio: 1.0 };
            }
        }

        if state.position == 0.0 && rsi < self.oversold {
            Signal::Buy { ratio: 1.0 }
        } else if state.position > 0.0 && rsi > self.target {
            Signal::Sell { ratio: 1.0 }
        } else {
            Signal::Hold
        }
    }

    fn reset(&mut self) {
        self.closes.clear();
    }
}
