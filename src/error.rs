use thiserror::Error;

#[derive(Error, Debug)]
pub enum BotError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Upbit API error: {code} - {message}")]
    Upbit { code: String, message: String },

    #[error("Config error: {0}")]
    Config(String),

    #[error("Strategy error: {0}")]
    Strategy(String),

    #[error("Order error: {0}")]
    Order(String),
}

pub type Result<T> = std::result::Result<T, BotError>;
