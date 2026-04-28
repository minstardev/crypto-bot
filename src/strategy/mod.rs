use crate::model::Candle;

pub mod buy_and_hold;

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
