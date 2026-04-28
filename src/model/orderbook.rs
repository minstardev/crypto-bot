use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookUnit {
    pub ask_price: f64,
    pub bid_price: f64,
    pub ask_size: f64,
    pub bid_size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orderbook {
    pub market: String,
    pub timestamp: i64,
    pub total_ask_size: f64,
    pub total_bid_size: f64,
    pub orderbook_units: Vec<OrderbookUnit>,
}
