use reqwest::Client;
use serde_json::{Value, json};
use std::time::Duration;

use crate::config::NotionConfig;
use crate::error::{BotError, Result};
use crate::report::table::StrategyReport;

const NOTION_VERSION: &str = "2022-06-28";
const NOTION_PAGES_URL: &str = "https://api.notion.com/v1/pages";

pub struct NotionReporter {
    http: Client,
    config: NotionConfig,
}

impl NotionReporter {
    pub fn new(config: NotionConfig) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .expect("failed to build http client");
        Self { http, config }
    }

    pub async fn upload(&self, report: &StrategyReport) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        let pf_value: Value = if report.metrics.profit_factor.is_finite() {
            json!(round2(report.metrics.profit_factor))
        } else {
            Value::Null
        };

        let mut body = json!({
            "parent": { "database_id": self.config.database_id },
            "properties": {
                "Strategy": { "title": [{ "text": { "content": report.result.strategy_name } }] },
                "Market": { "rich_text": [{ "text": { "content": report.result.market } }] },
                "Trades": { "number": report.metrics.trade_count },
                "WinRate": { "number": round2(report.metrics.win_rate) },
                "TotalReturn": { "number": round2(report.metrics.total_return) },
                "AnnReturn": { "number": round2(report.metrics.annualized_return) },
                "MaxDD": { "number": round2(report.metrics.max_drawdown) },
                "Sharpe": { "number": round2(report.metrics.sharpe) },
                "PF": { "number": pf_value },
                "FinalEquity": { "number": report.result.final_equity.round() },
                "RunAt": { "date": { "start": now } }
            }
        });

        if let Some(trend) = &report.trend {
            if let Some(props) = body.get_mut("properties").and_then(|v| v.as_object_mut()) {
                props.insert(
                    "Trend".into(),
                    json!({ "rich_text": [{ "text": { "content": trend } }] }),
                );
            }
        }

        let resp = self
            .http
            .post(NOTION_PAGES_URL)
            .bearer_auth(&self.config.token)
            .header("Notion-Version", NOTION_VERSION)
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(BotError::Notion(format!("{} - {}", status, text)));
        }

        Ok(())
    }

    pub async fn upload_all(&self, reports: &[StrategyReport]) -> Result<usize> {
        let mut ok = 0usize;
        for r in reports {
            match self.upload(r).await {
                Ok(()) => ok += 1,
                Err(e) => {
                    tracing::error!("Notion upload failed for {}: {}", r.result.strategy_name, e);
                }
            }
        }
        Ok(ok)
    }
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}
