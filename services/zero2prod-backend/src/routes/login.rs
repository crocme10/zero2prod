use axum::extract::{Json, State};
use axum::http::{header, status::StatusCode};
use axum::response::{IntoResponse, Response};
use axum_extra::extract::cookie::{Cookie, SameSite};
use hyper::header::HeaderMap;
use secrecy::Secret;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::fmt;
use uuid::Uuid;

use crate::authentication::jwt::build_token;
use crate::authentication::password::{Authenticator, Error as AuthenticationError};
use crate::domain::Credentials;
use crate::server::AppState;
use crate::storage::Error as StorageError;
use common::err_context::{ErrorContext, ErrorContextExt};

/// POST handler for user login
#[allow(clippy::unused_async)]
#[tracing::instrument(
    name = "User Login"
    skip(state),
    fields(
        request_id = %Uuid::new_v4(),
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let credentials = Credentials {
        username: request.username,
        password: Secret::new(request.password),
    };

    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    let authenticator = Authenticator {
        storage: state.storage.clone(),
    };

    let id = authenticator
        .validate_credentials(&credentials)
        .await
        .context("Could not validate credentials".to_string())?;

    let token = build_token(id, state.secret);

    let resp = LoginResp {
        status: "success".to_string(),
        token,
    };

    Ok::<_, Error>(resp)
}

/// This is what we return to the user in response to the subscription request.
/// Currently this is just a placeholder, and it does not return any useful
/// information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResp {
    pub status: String, // FIXME Placeholder
    pub token: String,
}

impl IntoResponse for LoginResp {
    fn into_response(self) -> Response {
        let LoginResp { status: _, token } = self.clone();
        let json = serde_json::to_string(&self).unwrap();
        let cookie = Cookie::build("jwt", token)
            .path("/")
            .max_age(time::Duration::hours(1))
            .same_site(SameSite::Lax)
            .http_only(true)
            .finish();
        let mut headers = HeaderMap::new();
        headers.insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
        (StatusCode::OK, headers, json).into_response()
    }
}

/// This is the information sent by the user to login.
#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub enum Error {
    InvalidCredentials {
        context: String,
        source: AuthenticationError,
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
            Error::InvalidCredentials { context, source } => {
                write!(fmt, "Invalid Credentials: {context} | {source}")
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

// impl From<ErrorContext<String, String>> for Error {
//     fn from(err: ErrorContext<String, String>) -> Self {
//         Error::InvalidRequest {
//             context: err.0,
//             source: err.1,
//         }
//     }
// }

impl From<ErrorContext<String, StorageError>> for Error {
    fn from(err: ErrorContext<String, StorageError>) -> Self {
        Error::Data {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<String, AuthenticationError>> for Error {
    fn from(err: ErrorContext<String, AuthenticationError>) -> Self {
        Error::InvalidCredentials {
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
            Error::InvalidCredentials { context, source: _ } => {
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
            err @ Error::InvalidCredentials { .. } => (
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
