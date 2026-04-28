use crate::api::auth::{build_query_string, build_token};
use crate::api::client::UpbitClient;
use crate::config::UpbitCredentials;
use crate::error::{BotError, Result};
use crate::model::{Account, Order, OrderRequest};

impl UpbitClient {
    fn credentials(&self) -> Result<&UpbitCredentials> {
        self.config
            .credentials
            .as_ref()
            .ok_or_else(|| BotError::Config("UPBIT_ACCESS_KEY / UPBIT_SECRET_KEY not set".into()))
    }

    pub async fn accounts(&self) -> Result<Vec<Account>> {
        let creds = self.credentials()?;
        let token = build_token(creds, None)?;
        let url = format!("{}/v1/accounts", self.config.base_url);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(token)
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<Account>>()
            .await?;
        Ok(resp)
    }

    pub async fn place_order(&self, req: &OrderRequest) -> Result<Order> {
        let creds = self.credentials()?;

        let mut params: Vec<(&str, String)> = vec![
            ("market", req.market.clone()),
            ("side", req.side.as_str().to_string()),
            ("ord_type", req.ord_type.as_str().to_string()),
        ];
        if let Some(v) = &req.volume {
            params.push(("volume", v.clone()));
        }
        if let Some(p) = &req.price {
            params.push(("price", p.clone()));
        }
        if let Some(id) = &req.identifier {
            params.push(("identifier", id.clone()));
        }

        let query = build_query_string(&params);
        let token = build_token(creds, Some(&query))?;
        let url = format!("{}/v1/orders", self.config.base_url);

        let resp = self
            .http
            .post(&url)
            .bearer_auth(token)
            .form(&params)
            .send()
            .await?
            .error_for_status()?
            .json::<Order>()
            .await?;
        Ok(resp)
    }

    pub async fn cancel_order(&self, uuid: &str) -> Result<Order> {
        let creds = self.credentials()?;
        let query = format!("uuid={}", uuid);
        let token = build_token(creds, Some(&query))?;
        let url = format!("{}/v1/order", self.config.base_url);
        let resp = self
            .http
            .delete(&url)
            .bearer_auth(token)
            .query(&[("uuid", uuid)])
            .send()
            .await?
            .error_for_status()?
            .json::<Order>()
            .await?;
        Ok(resp)
    }

    pub async fn order(&self, uuid: &str) -> Result<Order> {
        let creds = self.credentials()?;
        let query = format!("uuid={}", uuid);
        let token = build_token(creds, Some(&query))?;
        let url = format!("{}/v1/order", self.config.base_url);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(token)
            .query(&[("uuid", uuid)])
            .send()
            .await?
            .error_for_status()?
            .json::<Order>()
            .await?;
        Ok(resp)
    }
}
