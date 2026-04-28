use reqwest::Client;
use std::time::Duration;

use crate::config::Config;

#[derive(Clone)]
pub struct UpbitClient {
    pub(crate) http: Client,
    pub(crate) config: Config,
}

impl UpbitClient {
    pub fn new(config: Config) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("crypto-bot/0.1")
            .build()
            .expect("failed to build reqwest client");
        Self { http, config }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}
