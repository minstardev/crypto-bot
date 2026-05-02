use crate::error::Result;

#[derive(Debug, Clone)]
pub struct UpbitCredentials {
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Debug, Clone)]
pub struct NotionConfig {
    pub token: String,
    pub database_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveMode {
    Off,
    DryRun,
    Real,
}

#[derive(Debug, Clone)]
pub struct LiveConfig {
    pub mode: LiveMode,
    pub market: String,
    pub max_krw_per_trade: f64,
    pub exit_only: bool,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub credentials: Option<UpbitCredentials>,
    pub base_url: String,
    pub notion: Option<NotionConfig>,
    pub discord_webhook_long_term: Option<String>,
    pub discord_webhook_short_term: Option<String>,
    pub live: LiveConfig,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let _ = dotenvy::dotenv();

        let access_key = std::env::var("UPBIT_ACCESS_KEY").ok();
        let secret_key = std::env::var("UPBIT_SECRET_KEY").ok();

        let credentials = match (access_key, secret_key) {
            (Some(a), Some(s)) if !a.is_empty() && !s.is_empty() => {
                Some(UpbitCredentials { access_key: a, secret_key: s })
            }
            _ => None,
        };

        let notion_token = std::env::var("NOTION_TOKEN").ok();
        let notion_db = std::env::var("NOTION_DATABASE_ID").ok();
        let notion = match (notion_token, notion_db) {
            (Some(t), Some(d)) if !t.is_empty() && !d.is_empty() => {
                Some(NotionConfig { token: t, database_id: d })
            }
            _ => None,
        };

        let discord_webhook_long_term = std::env::var("DISCORD_WEBHOOK_LONG_TERM")
            .ok()
            .filter(|s| !s.is_empty());
        let discord_webhook_short_term = std::env::var("DISCORD_WEBHOOK_SHORT_TERM")
            .ok()
            .filter(|s| !s.is_empty());

        let live_mode = match std::env::var("LIVE_MODE")
            .ok()
            .as_deref()
            .map(|s| s.to_lowercase())
            .as_deref()
        {
            Some("dryrun") => LiveMode::DryRun,
            Some("real") => LiveMode::Real,
            _ => LiveMode::Off,
        };

        let exit_only = std::env::var("LIVE_EXIT_ONLY")
            .ok()
            .map(|s| matches!(s.to_lowercase().as_str(), "true" | "1" | "yes"))
            .unwrap_or(false);

        let live = LiveConfig {
            mode: live_mode,
            market: std::env::var("LIVE_MARKET")
                .unwrap_or_else(|_| "KRW-BTC".to_string()),
            max_krw_per_trade: std::env::var("LIVE_MAX_KRW_PER_TRADE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10_000.0),
            exit_only,
        };

        Ok(Self {
            credentials,
            base_url: std::env::var("UPBIT_BASE_URL")
                .unwrap_or_else(|_| "https://api.upbit.com".to_string()),
            notion,
            discord_webhook_long_term,
            discord_webhook_short_term,
            live,
        })
    }
}
