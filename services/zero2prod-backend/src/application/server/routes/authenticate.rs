use axum::extract::{Json, State};
use axum::http::{header, status::StatusCode, Request};
use axum::response::{IntoResponse, Response};
use axum_extra::extract::cookie::CookieJar;
use serde::Serialize;
use std::fmt;
use uuid::Uuid;

use crate::application::server::AppState;
use crate::authentication::jwt::{Authenticator, Error as JwtError};
use crate::domain::ports::secondary::AuthenticationError;
use common::err_context::{ErrorContext, ErrorContextExt};

/// GET handler for user authentication
#[allow(clippy::unused_async)]
#[tracing::instrument(
    name = "User Authentication"
    skip(state, cookie_jar),
    fields(
        request_id = %Uuid::new_v4(),
    )
)]
pub async fn authenticate<B: fmt::Debug>(
    cookie_jar: CookieJar,
    State(state): State<AppState>,
    req: Request<B>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let token = cookie_jar
        .get("token")
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

    let token = token.ok_or_else(|| Error::NotLoggedIn {
        context: "Unable to authenticate user".to_string(),
    })?;

    let authenticator = Authenticator {
        storage: state.authentication.clone(),
        secret: state.secret.clone(),
    };

    let id = authenticator
        .validate_token(&token)
        .await
        .context("Could not validate token")?;

    let resp = serde_json::json!({
        "status": "success",
        "id": id.to_string()
    });

    Ok::<_, Error>((
        StatusCode::OK,
        [
            ("X-Content-Type-Options", "nosniff"),
            ("X-Frame-Options", "DENY"),
            ("X-XSS-Protection", "0"),
            ("Cache-Control", "no-store"),
            (
                "Content-Security-Policy",
                "default-src 'none'; frame-ancestors 'none'; sandbox",
            ),
        ],
        Json(resp),
    ))
}

#[derive(Debug, Serialize)]
pub enum Error {
    InvalidCredentials {
        context: String,
        source: JwtError,
    },
    NotLoggedIn {
        context: String,
    },
    Data {
        context: String,
        source: AuthenticationError,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidCredentials { context, source } => {
                write!(fmt, "Invalid Credentials: {context} | {source}")
            }
            Error::NotLoggedIn { context } => {
                write!(fmt, "Not Logged In: {context} ")
            }
            Error::Data { context, source } => {
                write!(fmt, "Storage Error: {context} | {source}")
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

impl From<ErrorContext<JwtError>> for Error {
    fn from(err: ErrorContext<JwtError>) -> Self {
        Error::InvalidCredentials {
            context: err.0,
            source: err.1,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            err @ Error::InvalidCredentials { .. } => (
                StatusCode::UNAUTHORIZED,
                serde_json::to_string(&err).unwrap(),
            )
                .into_response(),
            err @ Error::NotLoggedIn { .. } => (
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
