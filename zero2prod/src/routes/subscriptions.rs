use axum::extract::{Json, State};
use axum_extra::extract::WithRejection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::NewSubscription;
use crate::email_service::Email;
use crate::error::ApiError;
use crate::server::AppState;

/// POST handler for user subscriptions
#[allow(clippy::unused_async)]
#[tracing::instrument(
    name = "Adding a new subscription"
    skip(state),
    fields(
        request_id = %Uuid::new_v4(),
    )
)]
pub async fn subscriptions(
    State(state): State<AppState>,
    WithRejection(Json(request), _): WithRejection<Json<SubscriptionRequest>, ApiError>,
) -> Result<Json<SubscriptionsResp>, ApiError> {
    let subscription = NewSubscription::try_from(request).map_err(ApiError::new_bad_request)?;

    state
        .storage
        .create_subscription(&subscription)
        .await
        .map_err(|err| ApiError::new_internal(format!("Cannot create new subscription: {err}")))?;

    let confirmation_link = format!("{}/subscription/confirmation", state.base_url);
    let html_content = format!(
        r#"Welcome to our newsletter!<br/> Click <a href="{}">here</a> to confirm your subscription"#,
        confirmation_link
    );
    let text_content = format!(
        r#"Welcome to our newsletter!\nVisit {} to confirm your subscription"#,
        confirmation_link
    );

    let email = Email {
        to: subscription.email,
        subject: "Welcome".to_string(),
        html_content,
        text_content,
    };

    state
        .email
        .send_email(email)
        .await
        .map_err(|err| ApiError::new_internal(format!("Cannot create new subscription: {err}")))?;

    tracing::debug!("Done");
    let resp = SubscriptionsResp {
        status: "OK".to_string(),
    };
    Ok(Json(resp))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionsResp {
    pub status: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubscriptionRequest {
    pub username: String,
    pub email: String,
}

#[cfg(test)]
mod tests {
    use crate::{
        domain::{NewSubscription, SubscriberEmail},
        email_service::MockEmailService,
        routes::subscriptions::SubscriptionRequest,
        server::{AppState, ApplicationBaseUrl},
        storage::MockStorage,
    };

    use super::*;
    use axum::{
        body::Body,
        http::{header, Request, StatusCode},
        routing::{post, Router},
    };
    use mockall::predicate::*;
    // if need to check response body: use serde_json::{json, Value};
    use std::sync::Arc;
    use tower::ServiceExt;

    fn subscription_route() -> Router<AppState> {
        Router::new().route("/subscriptions", post(subscriptions))
    }

    fn send_subscription_request(uri: &str, request: SubscriptionRequest) -> Request<Body> {
        Request::builder()
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .method("POST")
            .body(Body::from(
                serde_json::to_string(&request).expect("request"),
            ))
            .unwrap()
    }

    fn get_url_link(s: &str) -> String {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    }

    #[tokio::test]
    async fn subscription_should_store_subscriber_info() {
        let request = SubscriptionRequest {
            username: "bob".to_string(),
            email: "bob@acme.inc".to_string(),
        };

        let subscription = NewSubscription::try_from(request.clone()).unwrap();

        let mut storage_mock = MockStorage::new();
        storage_mock
            .expect_create_subscription()
            .with(eq(subscription))
            .return_once(|_| Ok(()));

        let mut email_mock = MockEmailService::new();
        email_mock.expect_send_email().return_once(|_| Ok(()));

        let state = AppState {
            storage: Arc::new(storage_mock),
            email: Arc::new(email_mock),
            base_url: ApplicationBaseUrl("http://127.0.0.1".to_string()),
        };

        let app = subscription_route().with_state(state);

        let response = app
            .oneshot(send_subscription_request("/subscriptions", request))
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
    async fn subscription_should_send_email_confirmation() {
        let base_url = "http://127.0.0.1".to_string();

        let request = SubscriptionRequest {
            username: "bob".to_string(),
            email: "bob@acme.inc".to_string(),
        };

        let mut email_mock = MockEmailService::new();
        email_mock
            .expect_send_email()
            .withf(|email: &Email| {
                let Email {
                    to,
                    subject: _,
                    html_content,
                    text_content: _,
                } = email;

                if *to != SubscriberEmail::parse("bob@acme.inc".to_string()).unwrap() {
                    return false;
                }
                let confirmation_link = get_url_link(html_content);
                println!("confirmation link: {confirmation_link}");
                if confirmation_link != "http://127.0.0.1/subscription/confirmation" {
                    return false;
                }
                true
            })
            .return_once(|_| Ok(()));

        // We also need a storage mock that returns 'Ok(())'
        let mut storage_mock = MockStorage::new();
        storage_mock
            .expect_create_subscription()
            .return_once(|_| Ok(()));

        let state = AppState {
            storage: Arc::new(storage_mock),
            email: Arc::new(email_mock),
            base_url: ApplicationBaseUrl(base_url),
        };

        let app = subscription_route().with_state(state);

        let response = app
            .oneshot(send_subscription_request("/subscriptions", request))
            .await
            .expect("response");

        // Check the response status code.
        assert_eq!(response.status(), StatusCode::OK);
    }
}
