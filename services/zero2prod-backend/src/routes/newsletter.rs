use axum::extract::{Json, State};
use axum_extra::extract::WithRejection;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::domain::SubscriberEmail;
use crate::email_service::Email;
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
    let subscribers = state
        .storage
        .get_confirmed_subscribers_email()
        .await
        .map_err(|err| {
            ApiError::new_internal(format!("Cannot retrieve confirmed subscribers: {err}"))
        })?;

    for subscriber in subscribers {
        let email = create_newsletter_email(&subscriber.email, &request);
        state.email.send_email(email).await.map_err(|err| {
            ApiError::new_internal(format!("Cannot send newsletter email: {err}"))
        })?;
    }
    Ok(Json(()))
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
        http::{header, Request},
        routing::{post, Router},
    };
    use mockall::predicate::*;
    use speculoos::prelude::*;
    use std::sync::Arc;
    use tower::ServiceExt;

    use crate::{
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
            assert_that(&response.status().as_u16())
                .named(message)
                .is_equal_to(400);
        }
    }
}
