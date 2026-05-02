use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};
use reqwest::Client;
use serde_json::json;

use crate::error::{BotError, Result};
use crate::executor::paper::{Trade, TradeSide};

const COLOR_BUY: u32 = 5_763_719;
const COLOR_SELL: u32 = 15_158_332;

pub struct DiscordNotifier {
    http: Client,
    webhook_url: String,
}

impl DiscordNotifier {
    pub fn new(webhook_url: String) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("failed to build http client");
        Self { http, webhook_url }
    }

    pub async fn send_trade(
        &self,
        trade: &Trade,
        market: &str,
        strategy: &str,
        regime: Option<&str>,
    ) -> Result<()> {
        let (title, color, side_emoji) = match trade.side {
            TradeSide::Buy => (format!("{} BUY  {}", "🟢", market), COLOR_BUY, "🟢"),
            TradeSide::Sell => (format!("{} SELL {}", "🔴", market), COLOR_SELL, "🔴"),
        };
        let _ = side_emoji;

        let total = trade.price * trade.volume;
        let ts = ms_to_iso(trade.timestamp);

        let mut fields = vec![
            json!({ "name": "Price",  "value": format!("₩{}", fmt_int(trade.price)),       "inline": true }),
            json!({ "name": "Volume", "value": format!("{:.8}", trade.volume),             "inline": true }),
            json!({ "name": "Total",  "value": format!("₩{}", fmt_int(total)),             "inline": true }),
            json!({ "name": "Strategy", "value": strategy, "inline": false }),
        ];

        if let Some(r) = regime {
            fields.push(json!({ "name": "Regime", "value": r, "inline": true }));
        }

        if matches!(trade.side, TradeSide::Sell) {
            if let Some(entry) = trade.entry_price {
                if entry > 0.0 {
                    let pnl_pct = (trade.price - entry) / entry * 100.0;
                    let pnl_krw = (trade.price - entry) * trade.volume;
                    let sign = if pnl_pct >= 0.0 { "+" } else { "" };
                    fields.push(json!({
                        "name": "PnL",
                        "value": format!("{}{:.2}% ({}{}₩{})",
                            sign, pnl_pct,
                            if pnl_krw >= 0.0 { "+" } else { "-" },
                            "",
                            fmt_int(pnl_krw.abs())),
                        "inline": true
                    }));
                }
            }
        }

        let body = json!({
            "embeds": [{
                "title": title,
                "color": color,
                "fields": fields,
                "timestamp": ts,
            }]
        });

        let resp = self
            .http
            .post(&self.webhook_url)
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(BotError::Discord(format!("{} - {}", status, text)));
        }

        Ok(())
    }

    pub async fn send_text(&self, content: &str) -> Result<()> {
        let body = json!({ "content": content });
        let resp = self
            .http
            .post(&self.webhook_url)
            .json(&body)
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(BotError::Discord(format!("{} - {}", status, text)));
        }
        Ok(())
    }
}

fn ms_to_iso(ms: i64) -> String {
    let dt: DateTime<Utc> = Utc
        .timestamp_millis_opt(ms)
        .single()
        .unwrap_or_else(Utc::now);
    dt.to_rfc3339()
}

pub fn fmt_int(v: f64) -> String {
    let n = v.round() as i64;
    let s = n.abs().to_string();
    let mut out = String::new();
    let bytes = s.as_bytes();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(*b as char);
    }
    if n < 0 {
        format!("-{}", out)
    } else {
        out
    }
}
