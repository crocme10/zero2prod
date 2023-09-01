use axum::extract::Json;
use axum::http::status::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::Value;

use serde::Serialize;
use std::fmt;

use crate::application::server::context::ContextError;
use crate::authentication::password::Error as PasswordError;
use crate::domain::ports::secondary::AuthenticationError;
use common::err_context::ErrorContext;

#[derive(Clone, Debug, Serialize)]
pub enum Error {
    AuthenticationService {
        context: String,
        source: AuthenticationError,
    },
    Credentials {
        context: String,
        source: PasswordError,
    },
    // This occurs when the credentials are not present in the context
    Context {
        context: String,
        source: ContextError,
    },
    DuplicateEmail {
        context: String,
    },
    DuplicateUsername {
        context: String,
    },
    WeakPassword {
        context: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::AuthenticationService { context, source } => {
                write!(fmt, "Authentication Service: {context} {source}")
            }
            Error::Credentials { context, source } => {
                write!(fmt, "Credentials: {context} {source}")
            }
            Error::Context { context, source } => {
                write!(fmt, "Context: {context} {source}")
            }
            Error::DuplicateEmail { context } => {
                write!(fmt, "Duplicate email: {context} ")
            }
            Error::DuplicateUsername { context } => {
                write!(fmt, "Duplicate username: {context} ")
            }
            Error::WeakPassword { context } => {
                write!(fmt, "Weak password: {context} ")
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

impl From<ErrorContext<AuthenticationError>> for Error {
    fn from(err: ErrorContext<AuthenticationError>) -> Self {
        Error::AuthenticationService {
            context: err.0,
            source: err.1,
        }
    }
}

impl Error {
    pub fn standardize(&self) -> (StatusCode, Json<Value>) {
        match self {
            Error::AuthenticationService { context, source: _ } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": "fail",
                    "message": context,
                    "code": "auth/internal_error".to_string()
                })),
            ),
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
            Error::DuplicateEmail { context } => (
                StatusCode::CONFLICT,
                Json(serde_json::json!({
                    "status": "fail",
                    "message": context,
                    "code": "auth/duplicate_email"
                })),
            ),
            Error::DuplicateUsername { context } => (
                StatusCode::CONFLICT,
                Json(serde_json::json!({
                    "status": "fail",
                    "message": context,
                    "code": "auth/duplicate_username"
                })),
            ),
            Error::WeakPassword { context } => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "status": "fail",
                    "message": context,
                    "code": "auth/weak_password"
                })),
            )
        }
    }
}
