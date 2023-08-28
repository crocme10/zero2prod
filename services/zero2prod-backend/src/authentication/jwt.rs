use chrono::{Duration, Utc};
use jsonwebtoken::{decode, DecodingKey, Validation};
use jsonwebtoken::{encode, EncodingKey, Header};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::ports::secondary::{AuthenticationError, AuthenticationStorage};
use common::err_context::{ErrorContext, ErrorContextExt};

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

// TODO This should really be a trait and an implementation...
// validate_credentials could be a free function, but for mocking
// it should be either a struct or a trait.
pub struct Authenticator {
    pub storage: Arc<dyn AuthenticationStorage + Send + Sync>,
    pub secret: Secret<String>,
}

#[cfg_attr(test, mockall::automock)]
impl Authenticator {
    // FIXME Can we trace?
    // #[tracing::instrument(name = "Validating Credentials")]
    pub async fn validate_token(&self, token: &str) -> Result<Uuid, Error> {
        let claims = decode::<TokenClaims>(
            token,
            &DecodingKey::from_secret(self.secret.expose_secret().as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| Error::InvalidToken)?
        .claims;

        let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|_| Error::InvalidToken)?;

        if self
            .storage
            .id_exists(&user_id)
            .await
            .context("Could not check if the id exists".to_string())?
        {
            Ok(user_id)
        } else {
            Err(Error::InvalidToken)
        }
    }
}

#[derive(Debug, Serialize)]
pub enum Error {
    InvalidToken,
    Data {
        context: String,
        source: AuthenticationError,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidToken => {
                write!(fmt, "Invalid Token")
            }
            Error::Data { context, source } => {
                write!(fmt, "Authentication Error: {context} | {source}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<ErrorContext<AuthenticationError>> for Error {
    fn from(err: ErrorContext<AuthenticationError>) -> Self {
        Error::Data {
            context: err.0,
            source: err.1,
        }
    }
}
