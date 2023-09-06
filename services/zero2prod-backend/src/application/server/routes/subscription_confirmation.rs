use axum::extract::{Json, Query, State};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Error;

use crate::application::server::AppState;
use common::err_context::ErrorContextExt;

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
) -> Result<impl IntoResponse, Error> {
    let request = request.0;
    match state
        .subscription
        .get_subscriber_id_by_token(&request.token)
        .await
        .context("Could not get subscriber id by token")?
    {
        None => Err(Error::MissingToken {
            context: "Expected token".to_string(),
        }),
        Some(id) => {
            state
                .subscription
                .confirm_subscriber_by_id_and_delete_token(&id)
                .await
                .context("Could not confirm subscriber")?;
            Ok::<_, Error>(Json(serde_json::json!({
                "status": "success"
            })))
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubscriptionConfirmationRequest {
    pub token: String,
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware::{from_fn_with_state, map_response},
        routing::{post, Router},
    };
    use fake::Fake;
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
    };

    use super::*;

    /// This is a helper function to build an App with axum.
    fn subscriptions_confirmation_route(state: AppState) -> Router {
        Router::new()
            .route(
                "/api/subscriptions/confirmation",
                post(subscriptions_confirmation),
            )
            .layer(map_response(error))
            .layer(from_fn_with_state(state.clone(), resolve_context))
            .layer(CookieManagerLayer::new())
            .with_state(state)
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

        let app = subscriptions_confirmation_route(state);

        let response = app
            .oneshot(send_subscription_confirmation_request(
                "/api/subscriptions/confirmation",
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

        let app = subscriptions_confirmation_route(state);

        let response = app
            .oneshot(send_subscription_confirmation_request(
                "/api/subscriptions/confirmation",
                Some(token),
            ))
            .await
            .expect("response");

        // Check the response status code.
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
