use axum::extract::{Json, State};
use axum_extra::extract::WithRejection;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{NewSubscription, SubscriberEmail};
use crate::email_service::Email;
use crate::error::ApiError;
use crate::server::{AppState, ApplicationBaseUrl};

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

    let id = state
        .storage
        .create_subscription(&subscription)
        .await
        .map_err(|err| ApiError::new_internal(format!("Cannot create new subscription: {err}")))?;

    let token = generate_subscription_token();

    state
        .storage
        .store_confirmation_token(&id, &token)
        .await
        .map_err(|err| ApiError::new_internal(format!("Cannot create new subscription: {err}")))?;

    let email = create_confirmation_email(&state.base_url, &subscription.email, &token);

    state
        .email
        .send_email(email)
        .await
        .map_err(|err| ApiError::new_internal(format!("Cannot create new subscription: {err}")))?;

    let resp = SubscriptionsResp {
        status: "OK".to_string(),
    };
    Ok(Json(resp))
}

/// This is a helper function to create an email sent to the subscriber,
/// which contains a link he needs to use to confirm his subscription.
/// the url argument is the URL of the zero2prod server, and will be used
/// as the base for the confirmation link.
fn create_confirmation_email(url: &ApplicationBaseUrl, to: &SubscriberEmail, token: &str) -> Email {
    let confirmation_link = format!(
        "{}/subscriptions/confirmation?subscription_token={}",
        url, token
    );
    let html_content = format!(
        r#"Welcome to our newsletter!<br/> Click <a href="{}">here</a> to confirm your subscription"#,
        confirmation_link
    );
    let text_content = format!(
        r#"Welcome to our newsletter!\nVisit {} to confirm your subscription"#,
        confirmation_link
    );

    Email {
        to: to.clone(),
        subject: "Welcome".to_string(),
        html_content,
        text_content,
    }
}

/// This is what we return to the user in response to the subscription request.
/// Currently this is just a placeholder, and it does not return any useful
/// information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionsResp {
    pub status: String,
}

/// This is the information sent by the user to request a subscription.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubscriptionRequest {
    pub username: String,
    pub email: String,
}

/// Generates a token (32 Alphanumeric String)
fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(32)
        .collect()
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{header, Request, StatusCode},
        routing::{post, Router},
    };
    use fake::faker::internet::en::{IPv4, SafeEmail};
    use fake::faker::name::en::Name;
    // use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::Fake;
    use mockall::predicate::*;
    use std::sync::Arc;
    use tower::ServiceExt;

    use crate::{
        domain::{NewSubscription, SubscriberEmail},
        email_service::MockEmailService,
        routes::subscriptions::SubscriptionRequest,
        server::{AppState, ApplicationBaseUrl},
        storage::{Error as StorageError, MockStorage},
    };

    use super::*;

    /// This is a helper function to build an App with axum.
    fn subscription_route() -> Router<AppState> {
        Router::new().route("/subscriptions", post(subscriptions))
    }

    /// This is a helper function to build the content of the request
    /// to our subscription endpoint. Essentially, it wraps the content
    /// of the subscription request into a html request with the proper header.
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

    /// This is a helper function to extract a url from a text.
    /// It assumes that the text contains one and only one url.
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
        // In this test, we use a MockStorage, and we expect that
        // the subscription handler will trigger a call to Storage::create_subscription.
        // Note that we do not actually use a database and check that the subscription is stored in
        // there.

        let username = Name().fake::<String>();
        let email = SafeEmail().fake::<String>();

        let request = SubscriptionRequest { username, email };

        let subscription = NewSubscription::try_from(request.clone()).unwrap();

        let subscriber_id = Uuid::new_v4();
        let mut storage_mock = MockStorage::new();
        storage_mock
            .expect_create_subscription()
            .with(eq(subscription))
            .return_once(move |_| Ok(subscriber_id));

        storage_mock
            .expect_store_confirmation_token()
            .withf(move |id: &Uuid, _token: &str| id == &subscriber_id)
            .return_once(|_, _| Ok(()));

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
        // In this test, we make sure that the subscription handler calls
        // the EmailService::send_email
        // We also check some of the field values in the email.

        let username = Name().fake::<String>();
        let email_addr = SafeEmail().fake::<String>();

        let request = SubscriptionRequest {
            username,
            email: email_addr.clone(),
        };

        let base_url = format!("http://{}", IPv4().fake::<String>());
        let base_url_clone = base_url.clone();
        let mut email_mock = MockEmailService::new();
        email_mock
            .expect_send_email()
            .withf(move |email: &Email| {
                let Email {
                    to,
                    subject: _,
                    html_content,
                    text_content: _,
                } = email;

                if *to != SubscriberEmail::parse(email_addr.clone()).unwrap() {
                    return false;
                }
                let confirmation_link = get_url_link(html_content);
                println!("confirmation link: {confirmation_link}");
                let confirmation_link_pattern =
                    format!("{}/subscriptions/confirmation", base_url_clone);
                if !confirmation_link.starts_with(&confirmation_link_pattern) {
                    return false;
                }
                true
            })
            .return_once(|_| Ok(()));

        let subscriber_id = Uuid::new_v4();
        // We also need a storage mock that returns 'Ok(())'
        let mut storage_mock = MockStorage::new();
        storage_mock
            .expect_create_subscription()
            .return_once(move |_| Ok(subscriber_id));
        storage_mock
            .expect_store_confirmation_token()
            .withf(move |id: &Uuid, _token: &str| id == &subscriber_id)
            .return_once(|_, _| Ok(()));

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

    #[tokio::test]
    async fn subscription_should_return_an_error_if_storage_fails() {
        // In this test, we use a MockStorage, and we expect that
        // the subscription handler will trigger a call to Storage::create_subscription.
        // Note that we do not actually use a database and check that the subscription is stored in
        // there.

        let username = Name().fake::<String>();
        let email = SafeEmail().fake::<String>();

        let request = SubscriptionRequest { username, email };

        let subscription = NewSubscription::try_from(request.clone()).unwrap();

        // This mock storage returns an error which does not really makes
        // sense.
        let mut storage_mock = MockStorage::new();
        storage_mock
            .expect_create_subscription()
            .with(eq(subscription))
            .return_once(|_| {
                Err(StorageError::Database {
                    context: "subscription context".to_string(),
                    source: sqlx::Error::RowNotFound,
                })
            });

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
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        // Check the response body.
        // let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        // let body: Value = serde_json::from_slice(&body).unwrap();
        // assert_eq!(body, json!(&dummy_heroes));
    }
}
