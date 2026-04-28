use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub market: String,
    #[serde(default)]
    pub trade_date: Option<String>,
    #[serde(default)]
    pub trade_time: Option<String>,
    pub trade_price: f64,
    pub prev_closing_price: f64,
    pub change: String,
    pub change_price: f64,
    pub change_rate: f64,
    pub signed_change_price: f64,
    pub signed_change_rate: f64,
    pub trade_volume: f64,
    pub acc_trade_price: f64,
    pub acc_trade_price_24h: f64,
    pub acc_trade_volume: f64,
    pub acc_trade_volume_24h: f64,
    pub highest_52_week_price: f64,
    pub lowest_52_week_price: f64,
    pub timestamp: i64,
}
