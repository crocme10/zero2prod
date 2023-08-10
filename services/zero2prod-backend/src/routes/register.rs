use axum::extract::{Json, State};
use axum::http::{header, status::StatusCode};
use axum::response::{IntoResponse, Response};
use axum_extra::extract::cookie::{Cookie, SameSite};
use common::err_context::{ErrorContext, ErrorContextExt};
use hyper::header::HeaderMap;
use serde::ser::SerializeStruct;
use passwords::{scorer, analyzer};
use serde::{Deserialize, Serialize, Serializer};
use std::fmt;
use uuid::Uuid;
use secrecy::Secret;

use crate::server::AppState;
use crate::storage::Error as StorageError;
use crate::domain::Credentials;
use crate::authentication::jwt::build_token;

/// POST handler for user registration
#[allow(clippy::unused_async)]
#[tracing::instrument(
    name = "User Registration"
    skip(state),
    fields(
        request_id = %Uuid::new_v4(),
    )
)]
pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegistrationRequest>,
) -> Result<impl IntoResponse, Error> {
    if state
        .storage
        .email_exists(&request.email)
        .await
        .context("Could not check if the email exists".to_string())?
    {
        return Err(Error::DuplicateEmail {
            context: "Unable to register new user".to_string(),
        });
    }

    if state
        .storage
        .username_exists(&request.username)
        .await
        .context("Could not check if the username exists".to_string())?
    {
        return Err(Error::DuplicateEmail {
            context: "Unable to register new user".to_string(),
        });
    }

    let password_score = scorer::score(&analyzer::analyze(&request.password));
    if password_score < 90f64 {
        return Err(Error::WeakPassword {
            context: "Unable to register new user".to_string(),
        });

    }

    let credentials = Credentials {
        username: request.username,
        password: Secret::new(request.password),
    };

    let id = Uuid::new_v4();

    state
        .storage
        .store_credentials(id, &request.email, &credentials)
        .await
        .context("Could not store credentials".to_string())?;

    let token = build_token(id, state.secret);

    let resp = RegistrationResp {
        status: "success".to_string(),
        token,
        id: id.to_string()
    };

    Ok::<_, Error>(resp)
}

/// This is what we return to the user in response to the subscription request.
/// Currently this is just a placeholder, and it does not return any useful
/// information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResp {
    pub status: String, // FIXME Placeholder
    pub token: String,
    pub id: String,
}

impl IntoResponse for RegistrationResp {
    fn into_response(self) -> Response {
        let RegistrationResp { status: _, token, id: _ } = self.clone();
        let json = serde_json::to_string(&self).unwrap();
        let cookie = Cookie::build("token", token)
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
pub struct RegistrationRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug)]
pub enum Error {
    DuplicateEmail {
        context: String,
    },
    DuplicateUsername {
        context: String,
    },
    WeakPassword {
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
            Error::DuplicateEmail { context } => {
                write!(fmt, "Duplicate email: {context} ")
            }
            Error::DuplicateUsername { context } => {
                write!(fmt, "Duplicate username: {context} ")
            }
            Error::WeakPassword { context } => {
                write!(fmt, "Weak password: {context} ")
            }
            Error::Data { context, source } => {
                write!(fmt, "Storage Error: {context} | {source}")
            }
        }
    }
}

impl std::error::Error for Error {}

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
            Error::DuplicateEmail { context } => {
                state.serialize_field("description", context)?;
            }
            Error::DuplicateUsername { context } => {
                state.serialize_field("description", context)?;
            }
            Error::WeakPassword { context } => {
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
            Error::DuplicateEmail { context } => (
                StatusCode::CONFLICT,
                Json(serde_json::json!({
                    "status": "fail",
                    "message": context
                }))
            )
                .into_response(),
            Error::DuplicateUsername { context } => (
                StatusCode::CONFLICT,
                Json(serde_json::json!({
                    "status": "fail",
                    "message": context
                }))
            )
                .into_response(),
            Error::WeakPassword { context } => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "status": "fail",
                    "message": context
                }))
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
