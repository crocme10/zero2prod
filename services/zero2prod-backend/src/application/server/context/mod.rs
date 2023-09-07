mod error;

use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use std::convert;
use uuid::Uuid;

pub use self::error::Error;
use crate::application::server::routes::Error as RoutesError;

#[derive(Clone, Debug)]
pub struct Context {
    user_id: Option<Uuid>,
}

impl Context {
    pub fn root() -> Self {
        Context { user_id: None }
    }

    pub fn new(user_id: Option<Uuid>) -> Self {
        Self { user_id }
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
            .get::<Result<Context, Error>>()
            .ok_or(Error::TokenNotFound)
            .map(|r| r.clone())
            .and_then(convert::identity)
            .map_err(|e| RoutesError::Context {
                context: "Could not extract Context".to_string(),
                source: e,
            })
    }
}
