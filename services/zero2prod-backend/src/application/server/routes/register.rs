use axum::extract::{Json, State};
use axum::response::IntoResponse;
use common::err_context::ErrorContextExt;
use passwords::{analyzer, scorer};
use secrecy::Secret;
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;
use uuid::Uuid;

use super::Error;
use crate::application::server::cookies;
use crate::application::server::AppState;
use crate::authentication::jwt::build_token;
use crate::domain::Credentials;

/// POST handler for user registration
/// The user submits credentials and other information, which will be stored.
/// The response can be:
/// - On success (correctly stored, no duplicate, strong password, ...) => {
///     - The user is considered logged in
///     - { "status": "success", "id": ... } + cookie
/// - On
#[allow(clippy::unused_async)]
#[tracing::instrument(
    name = "User Registration"
    skip(state, cookies, request),
    fields(
        request_id = %Uuid::new_v4(),
    )
)]
pub async fn register(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(request): Json<RegistrationRequest>,
) -> Result<impl IntoResponse, Error> {
    println!("register");
    // Check for duplicates
    if state
        .authentication
        .email_exists(&request.email)
        .await
        .context("Could not check if the email exists")?
    {
        println!("duplicate email");
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
        println!("duplicate username");
        return Err(Error::DuplicateUsername {
            context: "Unable to register new user".to_string(),
        });
    }

    let password_score = scorer::score(&analyzer::analyze(&request.password));
    if password_score < 90f64 {
        println!("weak password");
        return Err(Error::WeakPassword {
            context: "Unable to register new user".to_string(),
        });
    }

    println!("checked");

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

    let token = build_token(id, &state.secret);

    cookies::set_token_cookie(&cookies, &token);

    let resp = RegistrationResp {
        status: "success".to_string(),
        id: id.to_string(),
    };

    Ok::<_, Error>(Json(resp))
}

/// This is what we return to the user in response to the subscription request.
/// Currently this is just a placeholder, and it does not return any useful
/// information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResp {
    pub status: String, // FIXME Placeholder
    pub id: String,
}

/// This is the information sent by the user to login.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{header, Request, StatusCode},
        middleware::{from_fn_with_state, map_response},
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
    use tower_cookies::CookieManagerLayer;

    use crate::{
        application::server::{
            middleware::resolve_context::resolve_context, middleware::response_map::error,
        },
        application::server::{AppState, ApplicationBaseUrl},
        domain::ports::secondary::{
            MockAuthenticationStorage, MockEmailService, MockSubscriptionStorage,
        },
        domain::Credentials,
    };

    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FailedRegistrationResp {
        pub status: String,
        pub message: String,
        pub code: String,
    }

    fn registration_route(state: AppState) -> Router {
        Router::new()
            .route("/api/register", post(register))
            .layer(map_response(error))
            .layer(from_fn_with_state(state.clone(), resolve_context))
            .layer(CookieManagerLayer::new())
            .with_state(state)
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

        let app = registration_route(state);

        let response = app
            .oneshot(send_registration_request("/api/register", request))
            .await
            .expect("response");

        println!("response: {:?}", response);
        // Check the response status code.
        assert_eq!(response.status(), StatusCode::OK);

        // Check the response has a cookie
        // TODO make more checks on the cookie
        assert!(response.headers().contains_key(header::SET_COOKIE))
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

        let app = registration_route(state);

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

        let app = registration_route(state);

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

        let app = registration_route(state);

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
