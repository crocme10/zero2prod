use axum::extract::{Json, Query, State};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::server::AppState;

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
) -> Result<Json<SubscriptionConfirmationResp>, ApiError> {
    let request = request.0;
    let id = state
        .storage
        .get_subscriber_id_by_token(&request.token)
        .await
        .map_err(|err| ApiError::new_internal(format!("Cannot get subscriber by id: {err}")))?;

    match id {
        None => Err(ApiError::new_unauthorized(
            "Could not confirm subscription".to_string(),
        )),
        Some(id) => {
            state
                .storage
                .confirm_subscriber_by_id_and_delete_token(&id)
                .await
                .map_err(|err| {
                    ApiError::new_internal(format!("Cannot confirm subscriber by id: {err}"))
                })?;
            let resp = SubscriptionConfirmationResp {
                status: "OK".to_string(),
            };
            Ok(Json(resp))
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

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::{post, Router},
    };
    use fake::Fake;
    use mockall::predicate::*;
    use std::sync::Arc;
    use tower::ServiceExt;

    use crate::{
        email_service::MockEmailService,
        server::{AppState, ApplicationBaseUrl},
        storage::MockStorage,
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
        // In this test, we use a MockStorage, and we expect that
        // the subscription confirmation handler will trigger a call to
        // Storage::get_subscriber_id_by_token, and then use that id to confirm the
        // subscriber.

        let token = 32.fake::<String>();
        let mut storage_mock = MockStorage::new();
        let id = Uuid::new_v4();
        storage_mock
            .expect_get_subscriber_id_by_token()
            .with(eq(token.clone()))
            .return_once(move |_| Ok(Some(id.clone())));
        storage_mock
            .expect_confirm_subscriber_by_id_and_delete_token()
            .with(eq(id))
            .return_once(|_| Ok(()));

        let email_mock = MockEmailService::new();

        let state = AppState {
            storage: Arc::new(storage_mock),
            email: Arc::new(email_mock),
            base_url: ApplicationBaseUrl("http://127.0.0.1".to_string()),
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
        // In this test, we use a MockStorage, and we expect that:
        // - Storage::get_subscriber_id_by_token will get called (it returns None to simulate no
        //   valid token was found)
        // - Storage::confirm_subscriber_by_id never to get called,

        // FIXME Automagick alert!
        // 32 is the length of the token.
        let token = 32.fake::<String>();
        let mut storage_mock = MockStorage::new();
        storage_mock
            .expect_get_subscriber_id_by_token()
            .with(eq(token.clone()))
            .return_once(move |_| Ok(None));
        storage_mock
            .expect_confirm_subscriber_by_id_and_delete_token()
            .never()
            .return_once(|_| Ok(()));

        let email_mock = MockEmailService::new();

        let state = AppState {
            storage: Arc::new(storage_mock),
            email: Arc::new(email_mock),
            base_url: ApplicationBaseUrl("http://127.0.0.1".to_string()),
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
