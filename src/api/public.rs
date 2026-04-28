use crate::api::client::UpbitClient;
use crate::error::Result;
use crate::model::{Candle, Market, Orderbook, Ticker};

impl UpbitClient {
    pub async fn markets(&self, is_details: bool) -> Result<Vec<Market>> {
        let url = format!("{}/v1/market/all", self.config.base_url);
        let resp = self
            .http
            .get(&url)
            .query(&[("isDetails", is_details.to_string())])
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<Market>>()
            .await?;
        Ok(resp)
    }

    pub async fn ticker(&self, markets: &[&str]) -> Result<Vec<Ticker>> {
        let url = format!("{}/v1/ticker", self.config.base_url);
        let m = markets.join(",");
        let resp = self
            .http
            .get(&url)
            .query(&[("markets", m.as_str())])
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<Ticker>>()
            .await?;
        Ok(resp)
    }

    pub async fn candles_minutes(
        &self,
        unit: u32,
        market: &str,
        count: u32,
        to: Option<&str>,
    ) -> Result<Vec<Candle>> {
        let url = format!("{}/v1/candles/minutes/{}", self.config.base_url, unit);
        let count_s = count.to_string();
        let mut req = self
            .http
            .get(&url)
            .query(&[("market", market), ("count", count_s.as_str())]);
        if let Some(t) = to {
            req = req.query(&[("to", t)]);
        }
        let resp = req
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<Candle>>()
            .await?;
        Ok(resp)
    }

    pub async fn candles_days(
        &self,
        market: &str,
        count: u32,
        to: Option<&str>,
    ) -> Result<Vec<Candle>> {
        let url = format!("{}/v1/candles/days", self.config.base_url);
        let count_s = count.to_string();
        let mut req = self
            .http
            .get(&url)
            .query(&[("market", market), ("count", count_s.as_str())]);
        if let Some(t) = to {
            req = req.query(&[("to", t)]);
        }
        let resp = req
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<Candle>>()
            .await?;
        Ok(resp)
    }

    pub async fn candles_weeks(
        &self,
        market: &str,
        count: u32,
        to: Option<&str>,
    ) -> Result<Vec<Candle>> {
        let url = format!("{}/v1/candles/weeks", self.config.base_url);
        let count_s = count.to_string();
        let mut req = self
            .http
            .get(&url)
            .query(&[("market", market), ("count", count_s.as_str())]);
        if let Some(t) = to {
            req = req.query(&[("to", t)]);
        }
        let resp = req
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<Candle>>()
            .await?;
        Ok(resp)
    }

    pub async fn orderbook(&self, markets: &[&str]) -> Result<Vec<Orderbook>> {
        let url = format!("{}/v1/orderbook", self.config.base_url);
        let m = markets.join(",");
        let resp = self
            .http
            .get(&url)
            .query(&[("markets", m.as_str())])
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<Orderbook>>()
            .await?;
        Ok(resp)
    }
}
