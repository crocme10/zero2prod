use axum::extract::{Json, State};
use axum::http::status::StatusCode;
use axum::response::{IntoResponse, Response};
use secrecy::Secret;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::fmt;
use uuid::Uuid;

use crate::server::AppState;
use crate::storage::Error as StorageError;
use common::err_context::ErrorContext;

/// POST handler for user login
#[allow(clippy::unused_async)]
#[tracing::instrument(
    name = "User Login"
    skip(_state),
    fields(
        request_id = %Uuid::new_v4(),
    )
)]
pub async fn login(
    State(_state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    Ok::<axum::Json<()>, Error>(Json(()))
}

/// This is what we return to the user in response to the subscription request.
/// Currently this is just a placeholder, and it does not return any useful
/// information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResp {
    pub data: String, // FIXME Placeholder
}

/// This is the information sent by the user to login.
#[derive(Debug, Clone)]
pub struct LoginRequest {
    pub username: String,
    pub email: Secret<String>,
}

#[derive(Debug)]
pub enum Error {
    InvalidRequest {
        context: String,
        source: String,
    },
    MissingToken {
        context: String,
    },
    Data {
        context: String,
        source: StorageError,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidRequest { context, source } => {
                write!(fmt, "Invalid Request: {context} | {source}")
            }
            Error::MissingToken { context } => {
                write!(fmt, "Invalid Authentication Scheme: {context} ")
            }
            Error::Data { context, source } => {
                write!(fmt, "Storage Error: {context} | {source}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<ErrorContext<String, String>> for Error {
    fn from(err: ErrorContext<String, String>) -> Self {
        Error::InvalidRequest {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<String, StorageError>> for Error {
    fn from(err: ErrorContext<String, StorageError>) -> Self {
        Error::Data {
            context: err.0,
            source: err.1,
        }
    }
}

/// FIXME This is an oversimplified serialization for the Error.
/// I had to do this because some fields (source) where not 'Serialize'
impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Error", 1)?;
        match self {
            Error::InvalidRequest { context, source: _ } => {
                state.serialize_field("description", context)?;
            }
            Error::MissingToken { context } => {
                state.serialize_field("description", context)?;
            }
            Error::Data { context, source: _ } => {
                state.serialize_field("description", context)?;
            }
        }
        state.end()
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            err @ Error::InvalidRequest { .. } => (
                StatusCode::BAD_REQUEST,
                serde_json::to_string(&err).unwrap(),
            )
                .into_response(),
            err @ Error::MissingToken { .. } => (
                StatusCode::UNAUTHORIZED,
                serde_json::to_string(&err).unwrap(),
            )
                .into_response(),
            err @ Error::Data { .. } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                serde_json::to_string(&err).unwrap(),
            )
                .into_response(),
        }
    }
}
