use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub market: String,
    pub korean_name: String,
    pub english_name: String,
    #[serde(default)]
    pub market_warning: Option<String>,
}
