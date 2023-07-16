use axum::extract::{Json, State};
use axum_extra::extract::WithRejection;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiError;
use crate::server::AppState;

/// POST handler for newsletter publishing
#[allow(clippy::unused_async)]
#[tracing::instrument(
    name = "Publishing a newsletter issue"
    skip(state),
    fields(
        request_id = %Uuid::new_v4(),
    )
)]
pub async fn publish_newsletter(
    State(state): State<AppState>,
    WithRejection(Json(request), _): WithRejection<Json<BodyData>, ApiError>,
    ) -> Result<Json<()>, ApiError> {
    Ok(Json(()))
}


#[derive(Debug, Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(Debug, Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{header, Request, StatusCode},
        routing::{post, Router},
    };
    // use fake::faker::internet::en::{IPv4, SafeEmail};
    // use fake::faker::name::en::Name;
    // use fake::faker::lorem::en::{Paragraph, Sentence};
    // use fake::Fake;
    use mockall::predicate::*;
    use std::sync::Arc;
    use tower::ServiceExt;
    use speculoos::prelude::*;

    use crate::{
        domain::{NewSubscription, SubscriberEmail, Subscription, SubscriptionStatus},
        email_service::MockEmailService,
        routes::subscriptions::SubscriptionRequest,
        server::{AppState, ApplicationBaseUrl},
        storage::{Error as StorageError, MockStorage},
    };

    use super::*;

    /// This is a helper function to build an App with axum.
    fn newsletter_route() -> Router<AppState> {
        Router::new().route("/api/newsletter", post(publish_newsletter))
    }

    /// This is a helper function to build the content of the request
    /// to our subscription endpoint. Essentially, it wraps the content
    /// of the subscription request into a html request with the proper header.
    fn send_newsletter_request_from_json(uri: &str, request: serde_json::Value) -> Request<Body> {
        Request::builder()
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
            let response = app
                .oneshot(send_newsletter_request_from_json("/api/newsletter", body))
                .await
                .expect("Failed to execute request.");
            assert_that(&response.status().as_u16()).named(message).is_equal_to(400);
        }
    }
}
