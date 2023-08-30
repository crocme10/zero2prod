use axum::extract::Json;
use axum::http::status::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::Value;

use serde::Serialize;
use std::fmt;

use crate::application::server::context::ContextError;
use crate::authentication::password::Error as PasswordError;
use common::err_context::ErrorContext;

#[derive(Clone, Debug, Serialize)]
pub enum Error {
    Credentials {
        context: String,
        source: PasswordError,
    },
    Context {
        context: String,
        source: ContextError,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Credentials { context, source } => {
                write!(fmt, "Credentials: {context} {source}")
            }
            Error::Context { context, source } => {
                write!(fmt, "Context: {context} {source}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        self.standardize().into_response()
    }
}

impl From<ErrorContext<ContextError>> for Error {
    fn from(err: ErrorContext<ContextError>) -> Self {
        Error::Context {
            context: err.0,
            source: err.1,
        }
    }
}

impl Error {
    pub fn standardize(&self) -> (StatusCode, Json<Value>) {
        match self {
            Error::Credentials { context, source: _ } => (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "status": "fail",
                    "message": context,
                    "code": "auth/invalid_credentials".to_string()
                })),
            ),
            Error::Context { context, source: _ } => (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "status": "fail",
                    "message": context,
                    "code": "auth/missing_credentials".to_string()
                })),
            ),
        }
    }
}
