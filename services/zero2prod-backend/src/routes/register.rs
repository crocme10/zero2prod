use axum::extract::{Json, State};
use axum::http::{header, status::StatusCode};
use axum::response::{IntoResponse, Response};
use axum_extra::extract::cookie::{Cookie, SameSite};
use common::err_context::{ErrorContext, ErrorContextExt};
use hyper::header::HeaderMap;
use passwords::{analyzer, scorer};
use secrecy::Secret;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

use crate::application::server::AppState;
use crate::authentication::jwt::build_token;
use crate::domain::ports::secondary::AuthenticationError;
use crate::domain::Credentials;

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
        .authentication
        .email_exists(&request.email)
        .await
        .context("Could not check if the email exists")?
    {
        return Err(Error::DuplicateEmail {
            context: "Unable to register new user".to_string(),
        });
    }

    if state
        .authentication
        .username_exists(&request.username)
        .await
        .context("Could not check if the username exists")?
    {
        return Err(Error::DuplicateUsername {
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
        .authentication
        .store_credentials(id, &request.email, &credentials)
        .await
        .context("Could not store credentials")?;

    let token = build_token(id, state.secret);

    let resp = RegistrationResp {
        status: "success".to_string(),
        token,
        id: id.to_string(),
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
        let RegistrationResp {
            status: _,
            token,
            id: _,
        } = self.clone();
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
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
        source: AuthenticationError,
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

impl From<ErrorContext<AuthenticationError>> for Error {
    fn from(err: ErrorContext<AuthenticationError>) -> Self {
        Error::Data {
            context: err.0,
            source: err.1,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::DuplicateEmail { context } => (
                StatusCode::CONFLICT,
                Json(serde_json::json!({
                    "status": "fail",
                    "message": context,
                    "code": "auth/duplicate_email"
                })),
            )
                .into_response(),
            Error::DuplicateUsername { context } => (
                StatusCode::CONFLICT,
                Json(serde_json::json!({
                    "status": "fail",
                    "message": context,
                    "code": "auth/duplicate_username"
                })),
            )
                .into_response(),
            Error::WeakPassword { context } => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "status": "fail",
                    "message": context,
                    "code": "auth/weak_password"
                })),
            )
                .into_response(),
            Error::Data { .. } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": "fail",
                    "message": "unexpected error",
                    "code": "auth/internal"
                })),
            )
                .into_response(),
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{header, Request, StatusCode},
        routing::{post, Router},
    };
    use fake::faker::{
        internet::en::{Password, SafeEmail},
        name::en::Name,
    };
    use fake::Fake;
    use hyper::body::HttpBody;
    use mockall::predicate::*;
    use secrecy::Secret;
    use std::sync::Arc;
    use tower::ServiceExt;

    use crate::{
        application::server::{AppState, ApplicationBaseUrl},
        domain::ports::secondary::{
            MockAuthenticationStorage, MockEmailService, MockSubscriptionStorage,
        },
        domain::Credentials,
        routes::register::RegistrationRequest,
    };

    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FailedRegistrationResp {
        pub status: String,
        pub message: String,
        pub code: String,
    }

    fn registration_route() -> Router<AppState> {
        Router::new().route("/api/register", post(register))
    }

    /// This is a helper function to build the content of the request
    /// to our registration endpoint. Essentially, it wraps the content
    /// of the registration request into a html request with the proper header.
    fn send_registration_request(uri: &str, request: RegistrationRequest) -> Request<Body> {
        Request::builder()
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::ACCEPT, "application/json")
            .method("POST")
            .body(Body::from(
                serde_json::to_string(&request).expect("request"),
            ))
            .unwrap()
    }

    #[tokio::test]
    async fn registration_should_store_credentials() {
        let username = Name().fake::<String>();
        let email = SafeEmail().fake::<String>();
        let password = Password(12..32).fake::<String>();

        let email_clone = email.clone();
        let username_clone = username.clone();

        let request = RegistrationRequest {
            username: username.clone(),
            email: email.clone(),
            password: password.clone(),
        };

        let _credentials = Credentials {
            username,
            password: Secret::new(password),
        };

        let _id = Uuid::new_v4();

        let mut authentication_mock = MockAuthenticationStorage::new();
        let subscription_mock = MockSubscriptionStorage::new();
        authentication_mock
            .expect_store_credentials()
            .return_once(move |_, _, _| Ok(()));
        authentication_mock
            .expect_email_exists()
            .withf(move |email: &str| email == email_clone)
            .return_once(|_| Ok(false));
        authentication_mock
            .expect_username_exists()
            .withf(move |username: &str| username == username_clone)
            .return_once(|_| Ok(false));

        let email_mock = MockEmailService::new();
        let state = AppState {
            authentication: Arc::new(authentication_mock),
            subscription: Arc::new(subscription_mock),
            email: Arc::new(email_mock),
            base_url: ApplicationBaseUrl("http://127.0.0.1".to_string()),
            secret: Secret::new("secret".to_string()),
        };

        let app = registration_route().with_state(state);

        let response = app
            .oneshot(send_registration_request("/api/register", request))
            .await
            .expect("response");

        // Check the response status code.
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn registration_should_fail_if_username_exists() {
        let username = Name().fake::<String>();
        let email = SafeEmail().fake::<String>();
        let password = Password(12..32).fake::<String>();

        let email_clone = email.clone();
        let username_clone = username.clone();

        let request = RegistrationRequest {
            username: username.clone(),
            email: email.clone(),
            password: password.clone(),
        };

        let mut authentication_mock = MockAuthenticationStorage::new();
        let subscription_mock = MockSubscriptionStorage::new();

        authentication_mock
            .expect_store_credentials()
            .never()
            .return_once(move |_, _, _| Ok(()));
        authentication_mock
            .expect_email_exists()
            .withf(move |email: &str| email == email_clone)
            .return_once(|_| Ok(false));
        authentication_mock
            .expect_username_exists()
            .withf(move |username: &str| username == username_clone)
            .return_once(|_| Ok(true));

        let email_mock = MockEmailService::new();
        let state = AppState {
            authentication: Arc::new(authentication_mock),
            subscription: Arc::new(subscription_mock),
            email: Arc::new(email_mock),
            base_url: ApplicationBaseUrl("http://127.0.0.1".to_string()),
            secret: Secret::new("secret".to_string()),
        };

        let app = registration_route().with_state(state);

        let mut response = app
            .oneshot(send_registration_request("/api/register", request))
            .await
            .expect("response");

        // Check the response status code.
        assert_eq!(response.status(), StatusCode::CONFLICT);

        // Check the response code
        // let mut data = Vec::with_capacity(expected_length);
        let mut data = Vec::new();
        while let Some(chunk) = response.data().await {
            data.extend(&chunk.unwrap());
        }
        let response: FailedRegistrationResp = serde_json::from_slice(&data).expect("json");
        assert_eq!(response.status, "fail");
        assert_eq!(response.code, "auth/duplicate_username");
    }

    #[tokio::test]
    async fn registration_should_fail_if_email_exists() {
        let username = Name().fake::<String>();
        let email = SafeEmail().fake::<String>();
        let password = Password(12..32).fake::<String>();

        let email_clone = email.clone();
        let username_clone = username.clone();

        let request = RegistrationRequest {
            username: username.clone(),
            email: email.clone(),
            password: password.clone(),
        };

        let mut authentication_mock = MockAuthenticationStorage::new();
        let subscription_mock = MockSubscriptionStorage::new();

        authentication_mock
            .expect_store_credentials()
            .never()
            .return_once(move |_, _, _| Ok(()));
        authentication_mock
            .expect_email_exists()
            .withf(move |email: &str| email == email_clone)
            .return_once(|_| Ok(true));
        authentication_mock
            .expect_username_exists()
            .withf(move |username: &str| username == username_clone)
            .return_once(|_| Ok(false));

        let email_mock = MockEmailService::new();
        let state = AppState {
            authentication: Arc::new(authentication_mock),
            subscription: Arc::new(subscription_mock),
            email: Arc::new(email_mock),
            base_url: ApplicationBaseUrl("http://127.0.0.1".to_string()),
            secret: Secret::new("secret".to_string()),
        };

        let app = registration_route().with_state(state);

        let mut response = app
            .oneshot(send_registration_request("/api/register", request))
            .await
            .expect("response");

        // Check the response status code.
        assert_eq!(response.status(), StatusCode::CONFLICT);

        // Check the response code
        // let mut data = Vec::with_capacity(expected_length);
        let mut data = Vec::new();
        while let Some(chunk) = response.data().await {
            data.extend(&chunk.unwrap());
        }
        let response: FailedRegistrationResp = serde_json::from_slice(&data).expect("json");
        assert_eq!(response.status, "fail");
        assert_eq!(response.code, "auth/duplicate_email");
    }

    #[tokio::test]
    async fn registration_should_fail_if_password_is_weak() {
        let username = Name().fake::<String>();
        let email = SafeEmail().fake::<String>();
        let password = "Secret123".to_string();

        let request = RegistrationRequest {
            username: username.clone(),
            email: email.clone(),
            password: password.clone(),
        };

        let mut authentication_mock = MockAuthenticationStorage::new();
        let subscription_mock = MockSubscriptionStorage::new();

        authentication_mock
            .expect_store_credentials()
            .never()
            .return_once(move |_, _, _| Ok(()));
        authentication_mock
            .expect_email_exists()
            .return_once(|_| Ok(false));
        authentication_mock
            .expect_username_exists()
            .return_once(|_| Ok(false));

        let email_mock = MockEmailService::new();
        let state = AppState {
            authentication: Arc::new(authentication_mock),
            subscription: Arc::new(subscription_mock),
            email: Arc::new(email_mock),
            base_url: ApplicationBaseUrl("http://127.0.0.1".to_string()),
            secret: Secret::new("secret".to_string()),
        };

        let app = registration_route().with_state(state);

        let mut response = app
            .oneshot(send_registration_request("/api/register", request))
            .await
            .expect("response");

        // Check the response status code.
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Check the response code
        // let mut data = Vec::with_capacity(expected_length);
        let mut data = Vec::new();
        while let Some(chunk) = response.data().await {
            data.extend(&chunk.unwrap());
        }
        let response: FailedRegistrationResp = serde_json::from_slice(&data).expect("json");
        assert_eq!(response.status, "fail");
        assert_eq!(response.code, "auth/weak_password");
    }
}
