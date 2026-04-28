use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub currency: String,
    pub balance: String,
    pub locked: String,
    pub avg_buy_price: String,
    pub avg_buy_price_modified: bool,
    pub unit_currency: String,
}

impl Account {
    pub fn balance_f64(&self) -> f64 {
        self.balance.parse().unwrap_or(0.0)
    }
    pub fn locked_f64(&self) -> f64 {
        self.locked.parse().unwrap_or(0.0)
    }
    pub fn avg_buy_price_f64(&self) -> f64 {
        self.avg_buy_price.parse().unwrap_or(0.0)
    }
}
