use crate::api::client::UpbitClient;
use crate::error::Result;
use crate::model::{Order, OrderRequest, OrderSide, OrderType};

pub struct LiveExecutor {
    pub client: UpbitClient,
    pub market: String,
}

impl LiveExecutor {
    pub fn new(client: UpbitClient, market: impl Into<String>) -> Self {
        Self {
            client,
            market: market.into(),
        }
    }

    pub async fn market_buy_krw(&self, krw_amount: f64) -> Result<Order> {
        let req = OrderRequest {
            market: self.market.clone(),
            side: OrderSide::Bid,
            ord_type: OrderType::Price,
            volume: None,
            price: Some(format!("{:.0}", krw_amount)),
            identifier: None,
        };
        self.client.place_order(&req).await
    }

    pub async fn market_sell_volume(&self, volume: f64) -> Result<Order> {
        let req = OrderRequest {
            market: self.market.clone(),
            side: OrderSide::Ask,
            ord_type: OrderType::Market,
            volume: Some(format!("{:.8}", volume)),
            price: None,
            identifier: None,
        };
        self.client.place_order(&req).await
    }

    pub async fn limit_buy(&self, price: f64, volume: f64) -> Result<Order> {
        let req = OrderRequest {
            market: self.market.clone(),
            side: OrderSide::Bid,
            ord_type: OrderType::Limit,
            volume: Some(format!("{:.8}", volume)),
            price: Some(format!("{:.8}", price)),
            identifier: None,
        };
        self.client.place_order(&req).await
    }

    pub async fn limit_sell(&self, price: f64, volume: f64) -> Result<Order> {
        let req = OrderRequest {
            market: self.market.clone(),
            side: OrderSide::Ask,
            ord_type: OrderType::Limit,
            volume: Some(format!("{:.8}", volume)),
            price: Some(format!("{:.8}", price)),
            identifier: None,
        };
        self.client.place_order(&req).await
    }
}
