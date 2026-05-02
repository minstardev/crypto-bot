use std::collections::VecDeque;

use super::indicators;
use super::{MarketState, Signal, Strategy};
use crate::model::Candle;

pub struct Pullback {
    name: String,
    long_period: usize,
    short_period: usize,
    take_profit_pct: f64,
    stop_loss_pct: f64,
    closes: VecDeque<f64>,
}

impl Pullback {
    pub fn new(
        long_period: usize,
        short_period: usize,
        take_profit_pct: f64,
        stop_loss_pct: f64,
    ) -> Self {
        let name = format!(
            "Pullback(L{},S{},tp{:.0}%,sl{:.0}%)",
            long_period,
            short_period,
            take_profit_pct * 100.0,
            stop_loss_pct * 100.0,
        );
        Self {
            name,
            long_period,
            short_period,
            take_profit_pct,
            stop_loss_pct,
            closes: VecDeque::with_capacity(long_period),
        }
    }

    pub fn default_50_10() -> Self {
        Self::new(50, 10, 0.02, 0.05)
    }
}

impl Strategy for Pullback {
    fn name(&self) -> &str {
        &self.name
    }

    fn on_candle(&mut self, candle: &Candle, state: &MarketState) -> Signal {
        let close = candle.close();
        if self.closes.len() == self.long_period {
            self.closes.pop_front();
        }
        self.closes.push_back(close);

        if self.closes.len() < self.long_period {
            return Signal::Hold;
        }

        let all: Vec<f64> = self.closes.iter().copied().collect();
        let long_sma = indicators::sma(&all);
        let short_sma = indicators::sma(&all[all.len() - self.short_period..]);

        if state.position > 0.0 && state.avg_buy_price > 0.0 {
            let pnl_pct = (close - state.avg_buy_price) / state.avg_buy_price;
            if pnl_pct >= self.take_profit_pct {
                return Signal::Sell { ratio: 1.0 };
            }
            if pnl_pct < -self.stop_loss_pct {
                return Signal::Sell { ratio: 1.0 };
            }
        }

        if state.position == 0.0 && close > long_sma && close <= short_sma {
            Signal::Buy { ratio: 1.0 }
        } else {
            Signal::Hold
        }
    }

    fn reset(&mut self) {
        self.closes.clear();
    }
}
