use super::{MarketState, Signal, Strategy};
use crate::model::Candle;

pub struct BuyAndHold {
    bought: bool,
}

impl BuyAndHold {
    pub fn new() -> Self {
        Self { bought: false }
    }
}

impl Default for BuyAndHold {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for BuyAndHold {
    fn name(&self) -> &str {
        "BuyAndHold"
    }

    fn on_candle(&mut self, _candle: &Candle, state: &MarketState) -> Signal {
        if !self.bought && state.cash > 0.0 {
            self.bought = true;
            Signal::Buy { ratio: 1.0 }
        } else {
            Signal::Hold
        }
    }

    fn reset(&mut self) {
        self.bought = false;
    }
}
