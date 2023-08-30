mod error;

use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use serde::Serialize;
use std::{convert, fmt};
use uuid::Uuid;

pub use self::error::Error;
use crate::application::server::routes::Error as RoutesError;
use crate::authentication::jwt::Error as JwtError;
use common::err_context::ErrorContext;

#[derive(Clone, Debug)]
pub struct Context {
    user_id: Option<Uuid>,
}

impl Context {
    pub fn root() -> Self {
        Context { user_id: None }
    }

    pub fn new(user_id: Option<Uuid>) -> Result<Self, Error> {
        if user_id.is_none() {
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
