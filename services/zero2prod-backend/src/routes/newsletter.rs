use axum::extract::{Json, State};
use axum::http::status::StatusCode;
use axum::response::{IntoResponse, Response};
use base64::{DecodeError, Engine};
use hyper::header::{self, HeaderMap, ToStrError};
use secrecy::Secret;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::{fmt, string::FromUtf8Error};
use uuid::Uuid;

use crate::domain::{Credentials, SubscriberEmail};
use crate::email_service::{Email, Error as EmailError};
use crate::server::AppState;
use crate::storage::Error as StorageError;
use common::err_context::{ErrorContext, ErrorContextExt};

/// POST handler for newsletter publishing
#[allow(clippy::unused_async)]
#[tracing::instrument(
    name = "Adding a new subscription"
    skip(state),
    fields(
        request_id = %Uuid::new_v4(),
        username=tracing::field::Empty,
        id=tracing::field::Empty,
    )
)]
pub async fn publish_newsletter(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(request): Json<BodyData>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let credentials = basic_authentication(&headers)?;

    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    let id = state
        .storage
        .validate_credentials(&credentials)
        .await
        .context("Could not validate credentials".to_string())?;

    tracing::Span::current().record("id", &tracing::field::display(&id));

    let subscribers = state
        .storage
        .get_confirmed_subscribers_email()
        .await
        .context("Could not retrieve list of confirmed subscribers".to_string())?;

    for subscriber in subscribers {
        let email = create_newsletter_email(&subscriber.email, &request);
        state
            .email
            .send_email(email)
            .await
            .context("Cannot send newsletter email".to_string())?;
    }
    Ok::<axum::Json<()>, Error>(Json(()))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyData {
    pub title: String,
    pub content: Content,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    pub html: String,
    pub text: String,
}

#[derive(Debug)]
pub enum Error {
    MissingHeader {
        context: String,
    },
    InvalidAuthenticationString {
        context: String,
        source: ToStrError,
    },
    InvalidAuthenticationScheme {
        context: String,
    },
    Base64 {
        context: String,
        source: DecodeError,
    },
    CredentialString {
        context: String,
        source: FromUtf8Error,
    },
    InvalidCredentials {
        context: String,
    },
    Data {
        context: String,
        source: StorageError,
    },
    Email {
        context: String,
        source: EmailError,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MissingHeader { context } => {
                write!(fmt, "Missing Authentication: {context} ")
            }
            Error::InvalidAuthenticationString { context, source } => {
                write!(fmt, "Invalid Authentication Content: {context} | {source}")
            }
            Error::InvalidAuthenticationScheme { context } => {
                write!(fmt, "Invalid Authentication Scheme: {context} ")
            }
            Error::Base64 { context, source } => {
                write!(fmt, "Base64 Decode: {context} | {source}")
            }
            Error::CredentialString { context, source } => {
                write!(fmt, "Invalid UTF8 String: {context} | {source}")
            }
            Error::InvalidCredentials { context } => {
                write!(fmt, "Invalid Credentials: {context} ")
            }
            Error::Data { context, source } => {
                write!(fmt, "Storage Error: {context} | {source}")
            }
            Error::Email { context, source } => {
                write!(fmt, "Email Error: {context} | {source}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<ErrorContext<String, ToStrError>> for Error {
    fn from(err: ErrorContext<String, ToStrError>) -> Self {
        Error::InvalidAuthenticationString {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<String, DecodeError>> for Error {
    fn from(err: ErrorContext<String, DecodeError>) -> Self {
        Error::Base64 {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<String, FromUtf8Error>> for Error {
    fn from(err: ErrorContext<String, FromUtf8Error>) -> Self {
        Error::CredentialString {
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

impl From<ErrorContext<String, EmailError>> for Error {
    fn from(err: ErrorContext<String, EmailError>) -> Self {
        Error::Email {
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
            Error::MissingHeader { context } => {
                state.serialize_field("description", context)?;
            }
            Error::InvalidAuthenticationString { context, source: _ } => {
                state.serialize_field("description", context)?;
            }
            Error::InvalidAuthenticationScheme { context } => {
                state.serialize_field("description", context)?;
            }
            Error::Base64 { context, source: _ } => {
                state.serialize_field("description", context)?;
            }
            Error::CredentialString { context, source: _ } => {
                state.serialize_field("description", context)?;
            }
            Error::InvalidCredentials { context } => {
                state.serialize_field("description", context)?;
            }
            Error::Data { context, source: _ } => {
                state.serialize_field("description", context)?;
            }
            Error::Email { context, source: _ } => {
                state.serialize_field("description", context)?;
            }
        }
        state.end()
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (
            // FIXME Not all Error leads to UNAUTHORIZED. Some are INTERNAL_ERROR, ...
            StatusCode::UNAUTHORIZED,
            [
                (header::CONTENT_TYPE, "application/json"),
                (header::WWW_AUTHENTICATE, r#"Basic realm="publish""#),
            ],
            serde_json::to_string(&self).unwrap(),
        )
            .into_response()
    }
}

fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, Error> {
    // The header value, if present, must be a valid UTF8 string
    let header_value = headers
        .get("Authorization")
        .ok_or_else(|| Error::MissingHeader {
            context: "The Authorization Header was missing".to_string(),
        })?
        .to_str()
        .context("The 'Authorization' header was not a valid UTF8 string.".to_string())?;

    let base64encoded_segment =
        header_value
            .strip_prefix("Basic ")
            .ok_or_else(|| Error::InvalidAuthenticationScheme {
                context: "The Authorization scheme was not 'Basic'".to_string(),
            })?;

    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(base64encoded_segment)
        .context("Failed to base64-decode 'Basic' credentials.".to_string())?;

    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("The decoded credential string is not valid UTF8.".to_string())?;

    // Split into two segments, using ':' as delimiter
    let mut credentials = decoded_credentials.splitn(2, ':');
    let username = credentials
        .next()
        .ok_or_else(|| Error::InvalidCredentials {
            context: "The username was missing in 'Basic' authorization".to_string(),
        })?
        .to_string();
    let password = credentials
        .next()
        .ok_or_else(|| Error::InvalidCredentials {
            context: "The password was missing in 'Basic' authorization".to_string(),
        })?
        .to_string();
    Ok(Credentials {
        username,
        password: Secret::new(password),
    })
}

/// This is a helper function to create an email sent to the subscriber,
/// which contains a link he needs to use to confirm his subscription.
/// the url argument is the URL of the zero2prod server, and will be used
/// as the base for the confirmation link.
fn create_newsletter_email(to: &SubscriberEmail, newsletter: &BodyData) -> Email {
    let BodyData { title, content } = newsletter.clone();
    Email {
        to: to.clone(),
        subject: title.clone(),
        html_content: content.html.clone(),
        text_content: content.text.clone(),
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{header, Request, StatusCode},
        routing::{post, Router},
    };
    use fake::faker::internet::en::SafeEmail;
    use fake::locales::*;
    use fake::Fake;
    use mockall::predicate::*;
    use reqwest::header::HeaderValue;
    use speculoos::prelude::*;
    use std::sync::Arc;
    use tower::ServiceExt;

    use crate::{
        domain::{ConfirmedSubscriber, Credentials, SubscriberEmail, C},
        email_service::MockEmailService,
        server::{AppState, ApplicationBaseUrl},
        storage::MockStorage,
    };

    use super::*;

    /// This is a helper function to build an App with axum.
    fn newsletter_route() -> Router<AppState> {
        Router::new().route("/api/newsletter", post(publish_newsletter))
    }

    /// This is a helper function to build the content of the request
    /// to our subscription endpoint. Essentially, it wraps the content
    /// of the subscription request into a html request with the proper header.
    fn send_newsletter_request_from_json(
        uri: &str,
        request: serde_json::Value,
        credentials: Option<Credentials>,
    ) -> Request<Body> {
        let builder = match credentials {
            Some(credentials) => Request::builder().header(
                header::AUTHORIZATION,
                format!("Basic {}", credentials.encode()),
            ),
            None => Request::builder(),
        };
        builder
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .method("POST")
            .body(Body::from(request.to_string()))
            .unwrap()
    }

    #[tokio::test]
    async fn newsletters_returns_400_for_invalid_data() {
        // Arrange
        let storage_mock = MockStorage::new();
        let email_mock = MockEmailService::new();

        let state = AppState {
            storage: Arc::new(storage_mock),
            email: Arc::new(email_mock),
            base_url: ApplicationBaseUrl("http://127.0.0.1".to_string()),
        };

        let test_cases = vec![
            (
                serde_json::json!({
                    "content": {
                        "text": "Newsletter body as plain text",
                        "html": "<p>Newsletter body as HTML</p>",
                    }
                }),
                "missing title",
            ),
            (
                serde_json::json!({"title": "Newsletter!"}),
                "missing content",
            ),
        ];
        for (body, message) in test_cases {
            let app = newsletter_route().with_state(state.clone());
            //let credentials = Faker.fake::<C>();
            let credentials: Credentials = C(EN).fake();
            let response = app
                .oneshot(send_newsletter_request_from_json(
                    "/api/newsletter",
                    body,
                    Some(credentials),
                ))
                .await
                .expect("Failed to execute request.");
            assert_that(&response.status().as_u16())
                .named(message)
                .is_equal_to(422);
        }
    }

    #[tokio::test]
    async fn newsletter_should_send_email_notification() {
        // In this test, we make sure that the newsletter handler calls
        // the EmailService::send_email
        // We also check that the email recipient is that of the confirmed
        // subscriber in storage.

        // We create a fake email address. We will setup the mock storage
        // so that when we request a list of confirmed subscribers, this
        // email address is returned.
        let email_addr = SafeEmail().fake::<String>();

        let confirmed_subscriber = ConfirmedSubscriber {
            email: SubscriberEmail::try_from(email_addr.clone()).unwrap(),
        };

        let mut email_mock = MockEmailService::new();
        email_mock
            .expect_send_email()
            .withf(move |email: &Email| {
                let Email {
                    to,
                    subject: _,
                    html_content: _,
                    text_content: _,
                } = email;

                if *to != SubscriberEmail::parse(email_addr.clone()).unwrap() {
                    return false;
                }
                true
            })
            .return_once(|_| Ok(()));

        // We also need a storage mock that returns a list of confirmed subscribers
        let mut storage_mock = MockStorage::new();
        storage_mock
            .expect_get_confirmed_subscribers_email()
            .return_once(move || Ok(vec![confirmed_subscriber]));

        let credentials: Credentials = C(EN).fake();
        let rhs = credentials.username.clone();

        storage_mock
            .expect_validate_credentials()
            .withf(move |arg: &Credentials| {
                let Credentials {
                    username,
                    password: _,
                } = arg;
                if *username != rhs {
                    return false;
                }
                true
            })
            .return_once(|_| Ok(Uuid::new_v4()));

        let state = AppState {
            storage: Arc::new(storage_mock),
            email: Arc::new(email_mock),
            base_url: ApplicationBaseUrl("http://127.0.0.1".to_string()),
        };

        let app = newsletter_route().with_state(state);

        let body = BodyData {
            title: "Newsletter".to_string(),
            content: Content {
                html: "<p>Newsletter Content</p>".to_string(),
                text: "Newsletter Content".to_string(),
            },
        };
        let response = app
            .oneshot(send_newsletter_request_from_json(
                "/api/newsletter",
                serde_json::to_value(body).expect("body to json value"),
                Some(credentials),
            ))
            .await
            .expect("response");

        // Check the response status code.
        assert_that(&response.status()).is_equal_to(StatusCode::OK);
    }

    #[tokio::test]
    async fn newsletter_should_reject_request_without_authorization() {
        // In this test, we make sure that the newsletter handler does not call
        // the EmailService::send_email nor the StorageService, because
        // authorization criteria are not met

        let mut email_mock = MockEmailService::new();
        email_mock
            .expect_send_email()
            .never()
            .return_once(|_| Ok(()));
        let mut storage_mock = MockStorage::new();
        storage_mock
            .expect_get_confirmed_subscribers_email()
            .never()
            .return_once(|| Ok(vec![]));

        let state = AppState {
            storage: Arc::new(storage_mock),
            email: Arc::new(email_mock),
            base_url: ApplicationBaseUrl("http://127.0.0.1".to_string()),
        };

        let app = newsletter_route().with_state(state);

        let body = BodyData {
            title: "Newsletter".to_string(),
            content: Content {
                html: "<p>Newsletter Content</p>".to_string(),
                text: "Newsletter Content".to_string(),
            },
        };
        let response = app
            .oneshot(send_newsletter_request_from_json(
                "/api/newsletter",
                serde_json::to_value(body).expect("body to json value"),
                None,
            ))
            .await
            .expect("response");

        // Check the response status code.
        assert_that(&response.status()).is_equal_to(StatusCode::UNAUTHORIZED);
        assert_that(&response.headers()["WWW-Authenticate"])
            .is_equal_to(HeaderValue::from_static(r#"Basic realm="publish""#))
    }
}
