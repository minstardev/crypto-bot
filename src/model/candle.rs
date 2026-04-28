use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub market: String,
    pub candle_date_time_utc: String,
    pub candle_date_time_kst: String,
    pub opening_price: f64,
    pub high_price: f64,
    pub low_price: f64,
    pub trade_price: f64,
    pub timestamp: i64,
    pub candle_acc_trade_price: f64,
    pub candle_acc_trade_volume: f64,
    #[serde(default)]
    pub unit: Option<i32>,
    #[serde(default)]
    pub prev_closing_price: Option<f64>,
    #[serde(default)]
    pub change_price: Option<f64>,
    #[serde(default)]
    pub change_rate: Option<f64>,
}

impl Candle {
    pub fn open(&self) -> f64 { self.opening_price }
    pub fn high(&self) -> f64 { self.high_price }
    pub fn low(&self) -> f64 { self.low_price }
    pub fn close(&self) -> f64 { self.trade_price }
    pub fn volume(&self) -> f64 { self.candle_acc_trade_volume }
}
