use axum::extract::{Json, Query, State};
use axum::http::status::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::fmt;
use uuid::Uuid;

use crate::domain::ports::secondary::SubscriptionError;
use crate::server::AppState;
use common::err_context::{ErrorContext, ErrorContextExt};

/// POST handler for user subscription confirmation
#[allow(clippy::unused_async)]
#[tracing::instrument(
    name = "Confirming subscription with token"
    skip(state),
    fields(
        request_id = %Uuid::new_v4(),
    )
)]
pub async fn subscriptions_confirmation(
    State(state): State<AppState>,
    request: Query<SubscriptionConfirmationRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let request = request.0;
    match state
        .subscription
        .get_subscriber_id_by_token(&request.token)
        .await
        .context("Could not get subscriber id by token".to_string())?
    {
        None => Err(Error::MissingToken {
            context: "Expected token".to_string(),
        }),
        Some(id) => {
            state
                .subscription
                .confirm_subscriber_by_id_and_delete_token(&id)
                .await
                .context("Could not confirm subscriber".to_string())?;
            let resp = SubscriptionConfirmationResp {
                status: "OK".to_string(),
            };
            Ok::<axum::Json<SubscriptionConfirmationResp>, Error>(Json(resp))
        }
    }
}

/// This is what we return to the user in response to the subscription request.
/// Currently this is just a placeholder, and it does not return any useful
/// information.
/// FIXME Share code with frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionConfirmationResp {
    pub status: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubscriptionConfirmationRequest {
    pub token: String,
}

#[derive(Debug)]
pub enum Error {
    MissingToken {
        context: String,
    },
    Data {
        context: String,
        source: SubscriptionError,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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

impl From<ErrorContext<String, SubscriptionError>> for Error {
    fn from(err: ErrorContext<String, SubscriptionError>) -> Self {
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
            err @ Error::MissingToken { context: _ } => {
                (
                    // FIXME Not all Error leads to UNAUTHORIZED. Some are INTERNAL_ERROR, ...
                    StatusCode::UNAUTHORIZED,
                    serde_json::to_string(&err).unwrap(),
                )
                    .into_response()
            }
            err @ Error::Data {
                context: _,
                source: _,
            } => {
                (
                    // FIXME Not all Error leads to UNAUTHORIZED. Some are INTERNAL_ERROR, ...
                    StatusCode::INTERNAL_SERVER_ERROR,
                    serde_json::to_string(&err).unwrap(),
                )
                    .into_response()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::{post, Router},
    };
    use fake::Fake;
    use mockall::predicate::*;
    use secrecy::Secret;
    use std::sync::Arc;
    use tower::ServiceExt;

    use crate::{
        domain::ports::secondary::{
            MockAuthenticationStorage, MockEmailService, MockSubscriptionStorage,
        },
        server::{AppState, ApplicationBaseUrl},
    };

    use super::*;

    /// This is a helper function to build an App with axum.
    fn subscriptions_confirmation_route() -> Router<AppState> {
        Router::new().route(
            "/subscriptions/confirmation",
            post(subscriptions_confirmation),
        )
    }

    /// This is a helper function to build the content of the request
    /// to our subscription confirmation endpoint. Essentially, it wraps the conten
    /// of the subscription request into a html request with the proper header.
    fn send_subscription_confirmation_request(uri: &str, token: Option<String>) -> Request<Body> {
        let uri = format!(
            "{}{}",
            uri,
            token.map(|t| format!("?token={}", t)).unwrap_or_default()
        );
        Request::builder()
            .uri(uri)
            .method("POST")
            .body(Body::empty())
            .unwrap()
    }

    #[tokio::test]
    async fn subscription_confirmation_should_request_subscriber_info() {
        // In this test, we use a MockSubscriptionStorage, and we expect that
        // the subscription confirmation handler will trigger a call to
        // Storage::get_subscriber_id_by_token, and then use that id to confirm the
        // subscriber.

        let token = 32.fake::<String>();
        let authentication_mock = MockAuthenticationStorage::new();
        let mut subscription_mock = MockSubscriptionStorage::new();

        let id = Uuid::new_v4();
        subscription_mock
            .expect_get_subscriber_id_by_token()
            .with(eq(token.clone()))
            .return_once(move |_| Ok(Some(id)));
        subscription_mock
            .expect_confirm_subscriber_by_id_and_delete_token()
            .with(eq(id))
            .return_once(|_| Ok(()));

        let email_mock = MockEmailService::new();

        let state = AppState {
            authentication: Arc::new(authentication_mock),
            subscription: Arc::new(subscription_mock),
            email: Arc::new(email_mock),
            base_url: ApplicationBaseUrl("http://127.0.0.1".to_string()),
            secret: Secret::new("secret".to_string()),
        };

        let app = subscriptions_confirmation_route().with_state(state);

        let response = app
            .oneshot(send_subscription_confirmation_request(
                "/subscriptions/confirmation",
                Some(token),
            ))
            .await
            .expect("response");

        // Check the response status code.
        assert_eq!(response.status(), StatusCode::OK);

        // Check the response body.
        // let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        // let body: Value = serde_json::from_slice(&body).unwrap();
        // assert_eq!(body, json!(&dummy_heroes));
    }

    #[tokio::test]
    async fn subscription_confirmation_with_invalid_token_should_return_unauthorized() {
        // In this test, we use a MockSubscriptionStorage, and we expect that:
        // - Storage::get_subscriber_id_by_token will get called (it returns None to simulate no
        //   valid token was found)
        // - Storage::confirm_subscriber_by_id never to get called,

        // FIXME Automagick alert!
        // 32 is the length of the token.
        let token = 32.fake::<String>();
        let authentication_mock = MockAuthenticationStorage::new();
        let mut subscription_mock = MockSubscriptionStorage::new();

        subscription_mock
            .expect_get_subscriber_id_by_token()
            .with(eq(token.clone()))
            .return_once(move |_| Ok(None));
        subscription_mock
            .expect_confirm_subscriber_by_id_and_delete_token()
            .never()
            .return_once(|_| Ok(()));

        let email_mock = MockEmailService::new();

        let state = AppState {
            authentication: Arc::new(authentication_mock),
            subscription: Arc::new(subscription_mock),
            email: Arc::new(email_mock),
            base_url: ApplicationBaseUrl("http://127.0.0.1".to_string()),
            secret: Secret::new("secret".to_string()),
        };

        let app = subscriptions_confirmation_route().with_state(state);

        let response = app
            .oneshot(send_subscription_confirmation_request(
                "/subscriptions/confirmation",
                Some(token),
            ))
            .await
            .expect("response");

        // Check the response status code.
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
