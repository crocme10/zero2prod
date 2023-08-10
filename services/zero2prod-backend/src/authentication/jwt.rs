use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

pub fn build_token(id: Uuid, secret: Secret<String>) -> String {
    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::minutes(60)).timestamp() as usize;
    let claims = TokenClaims {
        sub: id.to_string(),
        exp,
        iat,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.expose_secret().as_bytes()),
    )
    .unwrap();

    token
}
