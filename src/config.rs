use crate::error::Result;

#[derive(Debug, Clone)]
pub struct UpbitCredentials {
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub credentials: Option<UpbitCredentials>,
    pub base_url: String,
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

        Ok(Self {
            credentials,
            base_url: std::env::var("UPBIT_BASE_URL")
                .unwrap_or_else(|_| "https://api.upbit.com".to_string()),
        })
    }
}
