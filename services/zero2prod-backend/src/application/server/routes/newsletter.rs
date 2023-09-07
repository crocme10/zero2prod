use axum::extract::{Json, State};
use axum::Extension;
use axum::response::IntoResponse;
use tower_cookies::Cookies;
use uuid::Uuid;

use super::Error;

use crate::application::server::{context::Context, AppState};
use crate::domain::ports::secondary::Email;
use crate::domain::BodyData;
use crate::domain::SubscriberEmail;
use common::err_context::ErrorContextExt;

/// POST handler for newsletter publishing
#[allow(clippy::unused_async)]
#[tracing::instrument(
    name = "Publishing a newsletter"
    skip(state),
    fields(
        request_id = %Uuid::new_v4(),
        username=tracing::field::Empty,
        id=tracing::field::Empty,
    )
)]
pub async fn publish_newsletter(
    Extension(context): Extension<Context>,
    State(state): State<AppState>,
    cookies: Cookies,
    Json(request): Json<BodyData>,
) -> Result<impl IntoResponse, Error> {
    let id = context
        .user_id()
        .map(|id| id.to_string())
        .unwrap_or("None".to_string());

    println!("context user id {}", id);

    tracing::Span::current().record("userid", &tracing::field::display(id));

    let subscribers = state
        .subscription
        .get_confirmed_subscribers_email()
        .await
        .context("Could not retrieve list of confirmed subscribers")?;

    for subscriber in subscribers {
        let email = create_newsletter_email(&subscriber.email, &request);
        state
            .email
            .send_email(email)
            .await
            .context("Cannot send newsletter email")?;
    }
    Ok::<axum::Json<()>, Error>(Json(()))
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
        middleware::{from_fn_with_state, map_response},
        routing::{post, Router},
    };
    use fake::faker::internet::en::SafeEmail;
    use fake::locales::*;
    use fake::Fake;
    use mockall::predicate::*;
    use reqwest::header::HeaderValue;
    use secrecy::Secret;
    use speculoos::prelude::*;
    use std::sync::Arc;
    use tower::ServiceExt;
    use tower_cookies::CookieManagerLayer;

    use crate::{
        application::server::{cookies::JWT, AppState, ApplicationBaseUrl},
        application::server::{
            middleware::resolve_context::resolve_context, middleware::response_map::error,
        },
        authentication::jwt::build_token,
        authentication::password::compute_password_hash,
        domain::ports::secondary::MockAuthenticationStorage,
        domain::ports::secondary::MockEmailService,
        domain::ports::secondary::MockSubscriptionStorage,
        domain::{
            ConfirmedSubscriber, Content, Credentials, CredentialsGenerator, SubscriberEmail,
        },
    };

    use super::*;

    /// This is a helper function to build an App with axum.
    fn newsletter_route(state: AppState) -> Router {
        Router::new()
            .route("/api/newsletter", post(publish_newsletter))
            .layer(map_response(error))
            .layer(from_fn_with_state(state.clone(), resolve_context))
            .layer(CookieManagerLayer::new())
            .with_state(state)
    }

    /// This is a helper function to build the content of the request
    /// to our subscription endpoint. Essentially, it wraps the content
    /// of the subscription request into a html request with the proper header.
    fn send_newsletter_request_from_json(
        uri: &str,
        request: serde_json::Value,
        id: Option<Uuid>,
        secret: &Secret<String>,
    ) -> Request<Body> {
        let builder = match id {
            Some(id) => {
                let token = build_token(id, &secret);
                Request::builder().header(header::COOKIE, format!("{}={}", JWT, token))
            }
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
    async fn newsletter_returns_400_for_invalid_data() {
        // Setup & Fixture

        // We create a fake user id, and make sure that the authentication mock believes it exists
        // in storage
        let user_id = Uuid::new_v4(); // This is the id of a user
        let mut authentication_mock = MockAuthenticationStorage::new();
        authentication_mock
            .expect_id_exists()
            .withf(move |id: &Uuid| id == &user_id)
            .return_const(Ok(true));

        let subscription_mock = MockSubscriptionStorage::new();
        let email_mock = MockEmailService::new();

        let state = AppState {
            authentication: Arc::new(authentication_mock),
            subscription: Arc::new(subscription_mock),
            email: Arc::new(email_mock),
            base_url: ApplicationBaseUrl("http://127.0.0.1".to_string()),
            secret: Secret::new("secret".to_string()),
        };

        // A list of <json = test content, string = test title>
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

        // Exec and Check
        for (body, message) in test_cases {
            let app = newsletter_route(state.clone());
            let response = app
                .oneshot(send_newsletter_request_from_json(
                    "/api/newsletter",
                    body,
                    Some(user_id),
                    &state.secret,
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
        let mut authentication_mock = MockAuthenticationStorage::new();
        let mut subscription_mock = MockSubscriptionStorage::new();

        subscription_mock
            .expect_get_confirmed_subscribers_email()
            .return_once(move || Ok(vec![confirmed_subscriber]));

        let credentials: Credentials = CredentialsGenerator(EN).fake();
        let hashed_password = compute_password_hash(credentials.password.clone()).unwrap();
        let id = Uuid::new_v4();
        authentication_mock
            .expect_get_credentials()
            .return_once(move |_| Ok(Some((id, hashed_password))));

        let state = AppState {
            authentication: Arc::new(authentication_mock),
            subscription: Arc::new(subscription_mock),
            email: Arc::new(email_mock),
            base_url: ApplicationBaseUrl("http://127.0.0.1".to_string()),
            secret: Secret::new("secret".to_string()),
        };

        let app = newsletter_route(state.clone());

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
                Some(id),
                &state.secret,
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
        let authentication_mock = MockAuthenticationStorage::new();
        let mut subscription_mock = MockSubscriptionStorage::new();

        subscription_mock
            .expect_get_confirmed_subscribers_email()
            .never()
            .return_once(|| Ok(vec![]));

        let state = AppState {
            authentication: Arc::new(authentication_mock),
            subscription: Arc::new(subscription_mock),
            email: Arc::new(email_mock),
            base_url: ApplicationBaseUrl("http://127.0.0.1".to_string()),
            secret: Secret::new("secret".to_string()),
        };

        let app = newsletter_route(state.clone());

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
                &state.secret,
            ))
            .await
            .expect("response");

        // Check the response status code.
        assert_that(&response.status()).is_equal_to(StatusCode::UNAUTHORIZED);
        assert_that(&response.headers()["WWW-Authenticate"])
            .is_equal_to(HeaderValue::from_static(r#"Basic realm="publish""#))
    }

    #[tokio::test]
    async fn newsletter_should_reject_request_with_invalid_credentials() {
        // In this test, we make sure that the newsletter handler does not call
        // the EmailService::send_email nor the StorageService, because
        // authorization criteria are not met.
        // So we use 'never' on the mocks.

        let mut email_mock = MockEmailService::new();
        email_mock
            .expect_send_email()
            .never()
            .return_once(|_| Ok(()));
        let mut authentication_mock = MockAuthenticationStorage::new();
        authentication_mock
            .expect_get_credentials()
            .return_once(move |_| Ok(None));
        let mut subscription_mock = MockSubscriptionStorage::new();

        subscription_mock
            .expect_get_confirmed_subscribers_email()
            .never()
            .return_once(|| Ok(vec![]));

        let state = AppState {
            authentication: Arc::new(authentication_mock),
            subscription: Arc::new(subscription_mock),
            email: Arc::new(email_mock),
            base_url: ApplicationBaseUrl("http://127.0.0.1".to_string()),
            secret: Secret::new("secret".to_string()),
        };

        let app = newsletter_route(state.clone());

        let body = BodyData {
            title: "Newsletter".to_string(),
            content: Content {
                html: "<p>Newsletter Content</p>".to_string(),
                text: "Newsletter Content".to_string(),
            },
        };
        let id = Uuid::new_v4();
        let response = app
            .oneshot(send_newsletter_request_from_json(
                "/api/newsletter",
                serde_json::to_value(body).expect("body to json value"),
                Some(id),
                &state.secret,
            ))
            .await
            .expect("response");

        // Check the response status code.
        assert_that(&response.status()).is_equal_to(StatusCode::UNAUTHORIZED);
        assert_that(&response.headers()["WWW-Authenticate"])
            .is_equal_to(HeaderValue::from_static(r#"Basic realm="publish""#))
    }
}
