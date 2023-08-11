use axum::extract::{Json, State};
use axum::http::{header, status::StatusCode};
use axum::response::{IntoResponse, Response};
use axum_extra::extract::cookie::{Cookie, SameSite};
use common::err_context::{ErrorContext, ErrorContextExt};
use hyper::header::HeaderMap;
use passwords::{analyzer, scorer};
use secrecy::Secret;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::fmt;
use uuid::Uuid;

use crate::authentication::jwt::build_token;
use crate::domain::Credentials;
use crate::server::AppState;
use crate::storage::Error as StorageError;

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
        state.serialize_field("status", "fail")?;
        match self {
            Error::DuplicateEmail { context } => {
                state.serialize_field("message", context)?;
            }
            Error::DuplicateUsername { context } => {
                state.serialize_field("message", context)?;
            }
            Error::WeakPassword { context } => {
                state.serialize_field("message", context)?;
            }
            Error::Data { context, source: _ } => {
                state.serialize_field("message", context)?;
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
                })),
            )
                .into_response(),
            Error::DuplicateUsername { context } => (
                StatusCode::CONFLICT,
                Json(serde_json::json!({
                    "status": "fail",
                    "message": context
                })),
            )
                .into_response(),
            Error::WeakPassword { context } => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "status": "fail",
                    "message": context
                })),
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
    use mockall::predicate::*;
    use secrecy::Secret;
    use std::sync::Arc;
    use tower::ServiceExt;

    use crate::{
        domain::Credentials,
        email_service::MockEmailService,
        routes::register::RegistrationRequest,
        server::{AppState, ApplicationBaseUrl},
        storage::MockStorage,
    };

    use super::*;

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
        // In this test, we use a MockStorage, and we expect that
        // the subscription handler will trigger a call to Storage::create_subscription.
        // Note that we do not actually use a database and check that the subscription is stored in
        // there.

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

        let mut storage_mock = MockStorage::new();
        storage_mock
            .expect_store_credentials()
            .return_once(move |_, _, _| Ok(()));
        storage_mock
            .expect_email_exists()
            .withf(move |email: &str| email == email_clone)
            .return_once(|_| Ok(false));
        storage_mock
            .expect_username_exists()
            .withf(move |username: &str| username == username_clone)
            .return_once(|_| Ok(false));

        let email_mock = MockEmailService::new();
        let state = AppState {
            storage: Arc::new(storage_mock),
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

    // #[tokio::test]
    // async fn subscription_should_send_email_confirmation() {
    //     // In this test, we make sure that the subscription handler calls
    //     // the EmailService::send_email
    //     // We also check some of the field values in the email.

    //     let username = Name().fake::<String>();
    //     let email_addr = SafeEmail().fake::<String>();

    //     let request = SubscriptionRequest {
    //         username,
    //         email: email_addr.clone(),
    //     };

    //     let new_subscription = NewSubscription::try_from(request.clone()).unwrap();
    //     let email_clone = new_subscription.email.clone();

    //     let base_url = format!("http://{}", IPv4().fake::<String>());
    //     let base_url_clone = base_url.clone();
    //     let mut email_mock = MockEmailService::new();
    //     email_mock
    //         .expect_send_email()
    //         .withf(move |email: &Email| {
    //             let Email {
    //                 to,
    //                 subject: _,
    //                 html_content,
    //                 text_content: _,
    //             } = email;

    //             if *to != SubscriberEmail::parse(email_addr.clone()).unwrap() {
    //                 return false;
    //             }
    //             let confirmation_link = get_url_link(html_content);
    //             println!("confirmation link: {confirmation_link}");
    //             let confirmation_link_pattern =
    //                 format!("{}/api/subscriptions/confirmation", base_url_clone);
    //             if !confirmation_link.starts_with(&confirmation_link_pattern) {
    //                 return false;
    //             }
    //             true
    //         })
    //         .return_once(|_| Ok(()));

    //     // We also need a storage mock that returns 'Ok(())'
    //     let mut storage_mock = MockStorage::new();
    //     storage_mock
    //         .expect_create_subscription_and_store_token()
    //         .return_once(move |_, _| {
    //             Ok(Subscription {
    //                 id: Uuid::new_v4(),
    //                 username: new_subscription.username,
    //                 email: new_subscription.email,
    //                 status: SubscriptionStatus::PendingConfirmation,
    //             })
    //         });

    //     storage_mock
    //         .expect_get_subscription_by_email()
    //         .withf(move |email: &str| email == email_clone.as_ref())
    //         .return_once(|_| Ok(None));

    //     let state = AppState {
    //         storage: Arc::new(storage_mock),
    //         email: Arc::new(email_mock),
    //         base_url: ApplicationBaseUrl(base_url),
    //         secret: Secret::new("secret".to_string()),
    //     };

    //     let app = subscription_route().with_state(state);

    //     let response = app
    //         .oneshot(send_subscription_request("/api/subscriptions", request))
    //         .await
    //         .expect("response");

    //     // Check the response status code.
    //     assert_eq!(response.status(), StatusCode::OK);
    // }

    // #[tokio::test]
    // async fn subscription_should_return_an_error_if_storage_fails() {
    //     // In this test, we use a MockStorage, and we expect that
    //     // the subscription handler will trigger a call to Storage::create_subscription.
    //     // Note that we do not actually use a database and check that the subscription is stored in
    //     // there.

    //     let username = Name().fake::<String>();
    //     let email = SafeEmail().fake::<String>();

    //     let request = SubscriptionRequest { username, email };

    //     let new_subscription = NewSubscription::try_from(request.clone()).unwrap();

    //     // This mock storage returns an error which does not really makes
    //     // sense.
    //     let mut storage_mock = MockStorage::new();
    //     storage_mock
    //         .expect_create_subscription_and_store_token()
    //         .withf(move |subscription: &NewSubscription, _token: &str| {
    //             subscription == &new_subscription
    //         })
    //         .return_once(|_, _| {
    //             Err(StorageError::Database {
    //                 context: "subscription context".to_string(),
    //                 source: sqlx::Error::RowNotFound,
    //             })
    //         });
    //     storage_mock
    //         .expect_get_subscription_by_email()
    //         .return_once(|_| Ok(None));

    //     let mut email_mock = MockEmailService::new();
    //     email_mock.expect_send_email().return_once(|_| Ok(()));

    //     let state = AppState {
    //         storage: Arc::new(storage_mock),
    //         email: Arc::new(email_mock),
    //         base_url: ApplicationBaseUrl("http://127.0.0.1".to_string()),
    //         secret: Secret::new("secret".to_string()),
    //     };

    //     let app = subscription_route().with_state(state);

    //     let response = app
    //         .oneshot(send_subscription_request("/api/subscriptions", request))
    //         .await
    //         .expect("response");

    //     // Check the response status code.
    //     assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    // }

    // #[tokio::test]
    // async fn subscription_should_return_error_if_invalid_data() {
    //     let username = Name().fake::<String>();
    //     let email = username.clone();

    //     let request = SubscriptionRequest { username, email };

    //     let storage_mock = MockStorage::new();

    //     let email_mock = MockEmailService::new();

    //     let state = AppState {
    //         storage: Arc::new(storage_mock),
    //         email: Arc::new(email_mock),
    //         base_url: ApplicationBaseUrl("http://127.0.0.1".to_string()),
    //         secret: Secret::new("secret".to_string()),
    //     };

    //     let app = subscription_route().with_state(state);

    //     let response = app
    //         .oneshot(send_subscription_request("/api/subscriptions", request))
    //         .await
    //         .expect("response");

    //     // Check the response status code.
    //     assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    // }
}
