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

    /// Fetch many minute candles via pagination. Returns chronological order (oldest first).
    pub async fn candles_minutes_paginated(
        &self,
        unit: u32,
        market: &str,
        total: usize,
    ) -> Result<Vec<Candle>> {
        const BATCH: u32 = 200;
        let mut acc: Vec<Candle> = Vec::with_capacity(total);
        let mut to: Option<String> = None;
        while acc.len() < total {
            let want = ((total - acc.len()) as u32).min(BATCH);
            let to_ref = to.as_deref();
            let mut batch = self.candles_minutes(unit, market, want, to_ref).await?;
            if batch.is_empty() {
                break;
            }
            // Upbit returns newest first. Use the oldest candle's KST time as next `to`.
            let oldest = batch.last().unwrap().clone();
            to = Some(oldest.candle_date_time_kst.replace('T', " "));
            acc.append(&mut batch);
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        }
        acc.sort_by_key(|c| c.timestamp);
        acc.dedup_by_key(|c| c.timestamp);
        Ok(acc)
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
