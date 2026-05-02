use std::collections::VecDeque;

use super::indicators;
use super::{MarketState, Signal, Strategy};
use crate::model::Candle;

pub struct BollingerReversion {
    name: String,
    period: usize,
    std_mul: f64,
    stop_loss_pct: f64,
    closes: VecDeque<f64>,
}

impl BollingerReversion {
    pub fn new(period: usize, std_mul: f64, stop_loss_pct: f64) -> Self {
        let name = format!(
            "Boll(P{},s{:.1},sl{:.0}%)",
            period,
            std_mul,
            stop_loss_pct * 100.0,
        );
        Self {
            name,
            period,
            std_mul,
            stop_loss_pct,
            closes: VecDeque::with_capacity(period),
        }
    }

    pub fn default_20() -> Self {
        Self::new(20, 2.0, 0.05)
    }
}

impl Strategy for BollingerReversion {
    fn name(&self) -> &str {
        &self.name
    }

    fn on_candle(&mut self, candle: &Candle, state: &MarketState) -> Signal {
        let close = candle.close();
        if self.closes.len() == self.period {
            self.closes.pop_front();
        }
        self.closes.push_back(close);

        if self.closes.len() < self.period {
            return Signal::Hold;
        }

        let prices: Vec<f64> = self.closes.iter().copied().collect();
        let bands = indicators::bollinger(&prices, self.std_mul);

        if state.position > 0.0 && state.avg_buy_price > 0.0 {
            let pnl_pct = (close - state.avg_buy_price) / state.avg_buy_price;
            if pnl_pct < -self.stop_loss_pct {
                return Signal::Sell { ratio: 1.0 };
            }
        }

        if state.position == 0.0 && close < bands.lower {
            Signal::Buy { ratio: 1.0 }
        } else if state.position > 0.0 && close >= bands.middle {
            Signal::Sell { ratio: 1.0 }
        } else {
            Signal::Hold
        }
    }

    fn reset(&mut self) {
        self.closes.clear();
    }
}
