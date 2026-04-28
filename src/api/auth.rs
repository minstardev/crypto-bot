use jsonwebtoken::{EncodingKey, Header, encode};
use serde::Serialize;
use sha2::{Digest, Sha512};
use uuid::Uuid;

use crate::config::UpbitCredentials;
use crate::error::Result;

#[derive(Debug, Serialize)]
struct Claims {
    access_key: String,
    nonce: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    query_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    query_hash_alg: Option<String>,
}

pub fn build_token(creds: &UpbitCredentials, query: Option<&str>) -> Result<String> {
    let (query_hash, query_hash_alg) = match query {
        Some(q) if !q.is_empty() => {
            let mut hasher = Sha512::new();
            hasher.update(q.as_bytes());
            (Some(hex::encode(hasher.finalize())), Some("SHA512".to_string()))
        }
        _ => (None, None),
    };

    let claims = Claims {
        access_key: creds.access_key.clone(),
        nonce: Uuid::new_v4().to_string(),
        query_hash,
        query_hash_alg,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(creds.secret_key.as_bytes()),
    )?;
    Ok(token)
}

pub fn build_query_string(params: &[(&str, String)]) -> String {
    params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&")
}
