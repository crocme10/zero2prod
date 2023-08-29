mod error;

pub use self::error::Error;
use crate::application::server::AppState;
use crate::authentication::jwt::{Authenticator, Error as JwtError};
use common::err_context::{ErrorContext, ErrorContextExt};

use crate::application::server::routes::Error as RoutesError;
use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::extract::State;
use axum::http::{header, request::Parts, Request};
use serde::Serialize;
use std::{convert, fmt};
use tower_cookies::Cookies;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Context {
    user_id: Option<Uuid>,
}

impl Context {
    pub fn root() -> Self {
        Context { user_id: None }
    }

    pub fn new(user_id: Option<Uuid>) -> Result<Self, Error> {
        if user_id == None {
            Err(Error::TBD {
                context: "Cannot assign to root context".to_string(),
            })
        } else {
            Ok(Self { user_id })
        }
    }
}

impl Context {
    pub fn user_id(&self) -> Option<Uuid> {
        self.user_id
    }
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Context {
    type Rejection = RoutesError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Result<Context, ContextError>>()
            .ok_or(ContextError::TokenNotFound)
            .map(|r| r.clone())
            .and_then(convert::identity)
            .map_err(|e| RoutesError::Context {
                context: "Could not extract Context".to_string(),
                source: e,
            })
    }
}

#[derive(Clone, Serialize, Debug)]
pub enum ContextError {
    TokenNotFound,
    InvalidCredentials { context: String, source: JwtError },
    InvalidUserId { context: String },
}

impl From<ErrorContext<JwtError>> for ContextError {
    fn from(err: ErrorContext<JwtError>) -> Self {
        ContextError::InvalidCredentials {
            context: err.0,
            source: err.1,
        }
    }
}

impl fmt::Display for ContextError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContextError::TokenNotFound => {
                write!(fmt, "Token not found")
            }
            ContextError::InvalidCredentials { context, source } => {
                write!(fmt, "Invalid Credentials: {context} {source}")
            }
            ContextError::InvalidUserId { context } => {
                write!(fmt, "Invalid User ID: {context}")
            }
        }
    }
}

#[allow(clippy::unused_async)]
#[tracing::instrument(
    name = "User Authentication"
    skip(state, cookies)
)]
pub async fn resolve_context<B: fmt::Debug>(
    cookies: &Cookies,
    State(state): State<AppState>,
    req: Request<B>,
) -> Result<Context, ContextError> {
    let token = cookies
        .get("jwt") // FIXME hardcoded
        .map(|cookie| cookie.value().to_string())
        .or_else(|| {
            req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth_header| auth_header.to_str().ok())
                .and_then(|auth_value| {
                    auth_value
                        .strip_prefix("Bearer ")
                        .map(|token| token.to_owned())
                })
        });

    let token = token.ok_or_else(|| ContextError::TokenNotFound)?;

    let authenticator = Authenticator {
        storage: state.authentication.clone(),
        secret: state.secret.clone(),
    };

    let id = authenticator
        .validate_token(&token)
        .await
        .context("Could not validate token")?;

    Context::new(Some(id)).map_err(|_| ContextError::InvalidUserId {
        context: "invalid user id".to_string(),
    })
}
