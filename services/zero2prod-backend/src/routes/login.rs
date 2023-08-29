use axum::extract::{Json, State};
use axum::http::{header, status::StatusCode, HeaderMap};
use axum::response::{IntoResponse, Response};
use axum_extra::extract::cookie::{Cookie, SameSite};
use secrecy::Secret;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

use crate::application::server::AppState;
use crate::authentication::jwt::build_token;
use crate::authentication::password::{Authenticator, Error as PasswordError};
use crate::domain::ports::secondary::AuthenticationError;
use crate::domain::Credentials;
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
        storage: state.authentication.clone(),
    };

    let id = authenticator
        .validate_credentials(&credentials)
        .await
        .context("Could not validate credentials")?;

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
        headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
        headers.insert("X-Frame-Options", "DENY".parse().unwrap());
        headers.insert("X-XSS-Protection", "0".parse().unwrap());
        headers.insert("Cache-Control", "no-store".parse().unwrap());
        headers.insert(
            "Content-Security-Policy",
            "default-src 'none'; frame-ancestors 'none'; sandbox"
                .parse()
                .unwrap(),
        );
        (StatusCode::OK, headers, json).into_response()
    }
}

/// This is the information sent by the user to login.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub enum Error {
    InvalidCredentials {
        context: String,
        source: PasswordError,
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

impl From<ErrorContext<PasswordError>> for Error {
    fn from(err: ErrorContext<PasswordError>) -> Self {
        Error::InvalidCredentials {
            context: err.0,
            source: err.1,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let mut headers = HeaderMap::new();
        headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
        headers.insert("X-Frame-Options", "DENY".parse().unwrap());
        headers.insert("X-XSS-Protection", "0".parse().unwrap());
        headers.insert("Cache-Control", "no-store".parse().unwrap());
        headers.insert(
            "Content-Security-Policy",
            "default-src 'none'; frame-ancestors 'none'; sandbox"
                .parse()
                .unwrap(),
        );
        match self {
            Error::InvalidCredentials { context, source: _ } => (
                StatusCode::UNAUTHORIZED,
                headers,
                Json(serde_json::json!({
                    "status": "fail",
                    "message": context,
                    "code": "auth/invalid_credentials"
                })),
            )
                .into_response(),
            Error::Data { context, source: _ } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                headers,
                Json(serde_json::json!({
                    "status": "fail",
                    "message": context,
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
    use fake::faker::{internet::en::Password, name::en::Name};
    use fake::Fake;
    use hyper::body::HttpBody;
    use mockall::predicate::*;
    use secrecy::Secret;
    use std::sync::Arc;
    use tower::ServiceExt;

    use crate::{
        application::server::{AppState, ApplicationBaseUrl},
        authentication::password::compute_password_hash,
        domain::ports::secondary::{
            MockAuthenticationStorage, MockEmailService, MockSubscriptionStorage,
        },
        routes::login::LoginRequest,
    };

    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FailedLoginResp {
        pub status: String,
        pub message: String,
        pub code: String,
    }

    fn login_route() -> Router<AppState> {
        Router::new().route("/api/login", post(login))
    }

    fn send_login_request(uri: &str, request: LoginRequest) -> Request<Body> {
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
    async fn login_should_retrieve_credentials() {
        let username = Name().fake::<String>();
        let password = Password(12..32).fake::<String>();
        let secret = Secret::new(password.clone());
        let password_hash = compute_password_hash(secret).expect("password hash");

        let request = LoginRequest {
            username: username.clone(),
            password: password.clone(),
        };

        let id = Uuid::new_v4();

        let mut authentication_mock = MockAuthenticationStorage::new();
        authentication_mock
            .expect_get_credentials()
            .return_once(move |_| Ok(Some((id, password_hash))));
        let subscription_mock = MockSubscriptionStorage::new();
        let email_mock = MockEmailService::new();
        let state = AppState {
            authentication: Arc::new(authentication_mock),
            subscription: Arc::new(subscription_mock),
            email: Arc::new(email_mock),
            base_url: ApplicationBaseUrl("http://127.0.0.1".to_string()),
            secret: Secret::new("secret".to_string()),
        };

        let app = login_route().with_state(state);

        let response = app
            .oneshot(send_login_request("/api/login", request))
            .await
            .expect("response");

        // Check the response status code.
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn login_should_fail_with_invalid_credentials() {
        let username = Name().fake::<String>();
        let password = Password(12..32).fake::<String>();
        let secret = Secret::new(password.clone());
        let password_hash = compute_password_hash(secret).expect("password hash");

        let request = LoginRequest {
            username: username.clone(),
            password: "secret".to_string(),
        };

        let id = Uuid::new_v4();

        let mut authentication_mock = MockAuthenticationStorage::new();
        authentication_mock
            .expect_get_credentials()
            .return_once(move |_| Ok(Some((id, password_hash))));
        let subscription_mock = MockSubscriptionStorage::new();

        let email_mock = MockEmailService::new();
        let state = AppState {
            authentication: Arc::new(authentication_mock),
            subscription: Arc::new(subscription_mock),
            email: Arc::new(email_mock),
            base_url: ApplicationBaseUrl("http://127.0.0.1".to_string()),
            secret: Secret::new("secret".to_string()),
        };

        let app = login_route().with_state(state);

        let mut response = app
            .oneshot(send_login_request("/api/login", request))
            .await
            .expect("response");

        // Check the response status code.
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Check the response code
        // let mut data = Vec::with_capacity(expected_length);
        let mut data = Vec::new();
        while let Some(chunk) = response.data().await {
            data.extend(&chunk.unwrap());
        }
        let response: FailedLoginResp = serde_json::from_slice(&data).expect("json");
        assert_eq!(response.status, "fail");
        assert_eq!(response.code, "auth/invalid_credentials");
    }
}
