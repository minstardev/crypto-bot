use crate::model::Candle;

pub mod bollinger_reversion;
pub mod buy_and_hold;
pub mod indicators;
pub mod pullback;
pub mod regime_switcher;
pub mod rsi_reversion;

pub use bollinger_reversion::BollingerReversion;
pub use buy_and_hold::BuyAndHold;
pub use pullback::Pullback;
pub use regime_switcher::RegimeSwitcher;
pub use rsi_reversion::RsiReversion;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Signal {
    Buy { ratio: f64 },
    Sell { ratio: f64 },
    Hold,
}

#[derive(Debug, Clone)]
pub struct MarketState {
    pub cash: f64,
    pub position: f64,
    pub avg_buy_price: f64,
    pub last_price: f64,
}

impl MarketState {
    pub fn equity(&self) -> f64 {
        self.cash + self.position * self.last_price
    }
    pub fn unrealized_pnl(&self) -> f64 {
        if self.position > 0.0 && self.avg_buy_price > 0.0 {
            (self.last_price - self.avg_buy_price) * self.position
        } else {
            0.0
        }
    }
}

pub trait Strategy {
    fn name(&self) -> &str;
    fn on_candle(&mut self, candle: &Candle, state: &MarketState) -> Signal;
    fn reset(&mut self) {}
}
