use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    Bid,
    Ask,
}

impl OrderSide {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderSide::Bid => "bid",
            OrderSide::Ask => "ask",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    Limit,
    Price,
    Market,
}

impl OrderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderType::Limit => "limit",
            OrderType::Price => "price",
            OrderType::Market => "market",
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrderRequest {
    pub market: String,
    pub side: OrderSide,
    pub ord_type: OrderType,
    pub volume: Option<String>,
    pub price: Option<String>,
    pub identifier: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Order {
    pub uuid: String,
    pub side: String,
    pub ord_type: String,
    #[serde(default)]
    pub price: Option<String>,
    pub state: String,
    pub market: String,
    pub created_at: String,
    #[serde(default)]
    pub volume: Option<String>,
    #[serde(default)]
    pub remaining_volume: Option<String>,
    #[serde(default)]
    pub reserved_fee: Option<String>,
    #[serde(default)]
    pub remaining_fee: Option<String>,
    #[serde(default)]
    pub paid_fee: Option<String>,
    #[serde(default)]
    pub locked: Option<String>,
    #[serde(default)]
    pub executed_volume: Option<String>,
    #[serde(default)]
    pub trades_count: Option<i64>,
}
