use crate::strategy::{MarketState, Signal};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
pub struct Trade {
    pub timestamp: i64,
    pub side: TradeSide,
    pub price: f64,
    pub volume: f64,
    pub fee: f64,
    pub cash_after: f64,
    pub position_after: f64,
}

#[derive(Debug, Clone)]
pub struct PaperExecutor {
    pub cash: f64,
    pub position: f64,
    pub avg_buy_price: f64,
    pub fee_rate: f64,
    pub slippage_rate: f64,
    pub trades: Vec<Trade>,
    pub equity_curve: Vec<(i64, f64)>,
}

impl PaperExecutor {
    pub fn new(initial_cash: f64) -> Self {
        Self {
            cash: initial_cash,
            position: 0.0,
            avg_buy_price: 0.0,
            fee_rate: 0.0005,
            slippage_rate: 0.0,
            trades: Vec::new(),
            equity_curve: Vec::new(),
        }
    }

    pub fn with_fee(mut self, fee_rate: f64) -> Self {
        self.fee_rate = fee_rate;
        self
    }

    pub fn with_slippage(mut self, slippage_rate: f64) -> Self {
        self.slippage_rate = slippage_rate;
        self
    }

    pub fn snapshot(&self, last_price: f64) -> MarketState {
        MarketState {
            cash: self.cash,
            position: self.position,
            avg_buy_price: self.avg_buy_price,
            last_price,
        }
    }

    pub fn equity(&self, last_price: f64) -> f64 {
        self.cash + self.position * last_price
    }

    pub fn apply(&mut self, signal: Signal, price: f64, ts: i64) {
        match signal {
            Signal::Buy { ratio } if ratio > 0.0 && self.cash > 0.0 => {
                let r = ratio.clamp(0.0, 1.0);
                let spend = self.cash * r;
                let fill = price * (1.0 + self.slippage_rate);
                let fee = spend * self.fee_rate;
                let buy_amount = if fill > 0.0 { (spend - fee) / fill } else { 0.0 };
                if buy_amount > 0.0 {
                    let new_position = self.position + buy_amount;
                    self.avg_buy_price = if new_position > 0.0 {
                        (self.avg_buy_price * self.position + fill * buy_amount) / new_position
                    } else {
                        0.0
                    };
                    self.position = new_position;
                    self.cash -= spend;
                    self.trades.push(Trade {
                        timestamp: ts,
                        side: TradeSide::Buy,
                        price: fill,
                        volume: buy_amount,
                        fee,
                        cash_after: self.cash,
                        position_after: self.position,
                    });
                }
            }
            Signal::Sell { ratio } if ratio > 0.0 && self.position > 0.0 => {
                let r = ratio.clamp(0.0, 1.0);
                let sell_volume = self.position * r;
                let fill = price * (1.0 - self.slippage_rate);
                let gross = sell_volume * fill;
                let fee = gross * self.fee_rate;
                self.cash += gross - fee;
                self.position -= sell_volume;
                if self.position <= 1e-12 {
                    self.position = 0.0;
                    self.avg_buy_price = 0.0;
                }
                self.trades.push(Trade {
                    timestamp: ts,
                    side: TradeSide::Sell,
                    price: fill,
                    volume: sell_volume,
                    fee,
                    cash_after: self.cash,
                    position_after: self.position,
                });
            }
            _ => {}
        }
        self.equity_curve.push((ts, self.equity(price)));
    }
}
