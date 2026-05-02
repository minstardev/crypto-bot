use super::{MarketState, Signal, Strategy};
use crate::model::Candle;

pub struct BuyAndHold;

impl BuyAndHold {
    pub fn new() -> Self {
        Self
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
        if state.position == 0.0 && state.cash > 0.0 {
            Signal::Buy { ratio: 1.0 }
        } else {
            Signal::Hold
        }
    }
}
