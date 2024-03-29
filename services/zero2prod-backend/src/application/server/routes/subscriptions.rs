use axum::extract::{Json, State};
use axum::response::IntoResponse;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Error;

use crate::application::server::{AppState, ApplicationBaseUrl};
use crate::domain::ports::secondary::Email;
use crate::domain::{
    NewSubscription, SubscriberEmail, Subscription, SubscriptionRequest, SubscriptionStatus,
};
use common::err_context::ErrorContextExt;

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
    Json(request): Json<SubscriptionRequest>,
) -> Result<impl IntoResponse, Error> {
    let subscription =
        NewSubscription::try_from(request).context("Could not get valid subscription")?;

    match state
        .subscription
        .get_subscription_by_email(subscription.email.as_ref())
        .await
        .context("Could not get subscription by email")?
    {
        None => {
            tracing::info!("No prior subscription found");
            let token = generate_subscription_token();

            let subscription = state
                .subscription
                .create_subscription_and_store_token(&subscription, &token)
                .await
                .context("Could not create new subscription")?;

            let email = create_confirmation_email(&state.base_url, &subscription.email, &token);

            state
                .email
                .send_email(email)
                .await
                .context("Could not send confirmation email")?;

            let resp = SubscriptionsResp { subscription };
            Ok::<axum::Json<SubscriptionsResp>, Error>(Json(resp))
        }
        Some(subscription) => {
            // FIXME The logic here is probably not very secure. It's not taking the
            // username into account, and more...
            // Depending on the subscription's status:
            // * if it is 'pending_confirmation', then we get the token, and send another
            //   confirmation email
            // * if it is 'confirmed', then we send an email 'already subscribed'
            match subscription.status {
                SubscriptionStatus::PendingConfirmation => {
                    match state
                        .subscription
                        .get_token_by_subscriber_id(&subscription.id)
                        .await
                        .context("Could not get token by subscriber's id")?
                    {
                        None => Err(Error::MissingToken {
                            context: "Expected token".to_string(),
                        }),
                        Some(token) => {
                            let email = create_confirmation_email(
                                &state.base_url,
                                &subscription.email,
                                &token,
                            );

                            state
                                .email
                                .send_email(email)
                                .await
                                .context("Could not send confirmation email")?;
                            let resp = SubscriptionsResp { subscription };
                            Ok::<axum::Json<SubscriptionsResp>, Error>(Json(resp))
                        }
                    }
                }
                SubscriptionStatus::Confirmed => {
                    let email = Email {
                        to: subscription.email.clone(),
                        subject: "Already Subscribed".to_string(),
                        html_content: "You are already subscribed".to_string(),
                        text_content: "You are already subscribed".to_string(),
                    };
                    state
                        .email
                        .send_email(email)
                        .await
                        .context("Could not send confirmation email")?;
                    let resp = SubscriptionsResp { subscription };
                    Ok::<axum::Json<SubscriptionsResp>, Error>(Json(resp))
                }
            }
        }
    }
}

/// This is a helper function to create an email sent to the subscriber,
/// which contains a link he needs to use to confirm his subscription.
/// the url argument is the URL of the zero2prod server, and will be used
/// as the base for the confirmation link.
fn create_confirmation_email(url: &ApplicationBaseUrl, to: &SubscriberEmail, token: &str) -> Email {
    let confirmation_link = format!("{}/api/subscriptions/confirmation?token={}", url, token);
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
    pub subscription: Subscription,
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
        middleware::{from_fn_with_state, map_response},
        routing::{post, Router},
    };
    use fake::faker::{
        internet::en::{IPv4, SafeEmail},
        name::en::Name,
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
            MockAuthenticationStorage, MockEmailService, MockSubscriptionStorage, SubscriptionError,
        },
        domain::{NewSubscription, SubscriberEmail, Subscription, SubscriptionStatus},
    };

    use super::*;

    /// This is a helper function to build an App with axum.
    fn subscription_route(state: AppState) -> Router {
        Router::new()
            .route("/api/subscriptions", post(subscriptions))
            .layer(map_response(error))
            .layer(from_fn_with_state(state.clone(), resolve_context))
            .layer(CookieManagerLayer::new())
            .with_state(state)
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

        let new_subscription = NewSubscription::try_from(request.clone()).unwrap();

        let username = new_subscription.username.clone();
        let email = new_subscription.email.clone();
        let email_clone = email.clone();
        let authentication_mock = MockAuthenticationStorage::new();
        let mut subscription_mock = MockSubscriptionStorage::new();

        subscription_mock
            .expect_create_subscription_and_store_token()
            .withf(move |subscription: &NewSubscription, _token: &str| {
                subscription == &new_subscription
            })
            .return_once(move |_, _| {
                Ok(Subscription {
                    id: Uuid::new_v4(),
                    username,
                    email,
                    status: SubscriptionStatus::PendingConfirmation,
                })
            });
        subscription_mock
            .expect_get_subscription_by_email()
            .withf(move |email: &str| email == email_clone.as_ref())
            .return_once(|_| Ok(None));

        let mut email_mock = MockEmailService::new();
        email_mock.expect_send_email().return_once(|_| Ok(()));

        let state = AppState {
            authentication: Arc::new(authentication_mock),
            subscription: Arc::new(subscription_mock),
            email: Arc::new(email_mock),
            base_url: ApplicationBaseUrl("http://127.0.0.1".to_string()),
            secret: Secret::new("secret".to_string()),
        };

        let app = subscription_route(state);

        let response = app
            .oneshot(send_subscription_request("/api/subscriptions", request))
            .await
            .expect("response");

        // Check the response status code.
        assert_eq!(response.status(), StatusCode::OK);
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

        let new_subscription = NewSubscription::try_from(request.clone()).unwrap();
        let email_clone = new_subscription.email.clone();

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
                    format!("{}/api/subscriptions/confirmation", base_url_clone);
                if !confirmation_link.starts_with(&confirmation_link_pattern) {
                    return false;
                }
                true
            })
            .return_once(|_| Ok(()));

        // We also need a storage mock that returns 'Ok(())'
        let authentication_mock = MockAuthenticationStorage::new();
        let mut subscription_mock = MockSubscriptionStorage::new();

        subscription_mock
            .expect_create_subscription_and_store_token()
            .return_once(move |_, _| {
                Ok(Subscription {
                    id: Uuid::new_v4(),
                    username: new_subscription.username,
                    email: new_subscription.email,
                    status: SubscriptionStatus::PendingConfirmation,
                })
            });

        subscription_mock
            .expect_get_subscription_by_email()
            .withf(move |email: &str| email == email_clone.as_ref())
            .return_once(|_| Ok(None));

        let state = AppState {
            authentication: Arc::new(authentication_mock),
            subscription: Arc::new(subscription_mock),
            email: Arc::new(email_mock),
            base_url: ApplicationBaseUrl(base_url),
            secret: Secret::new("secret".to_string()),
        };

        let app = subscription_route(state);

        let response = app
            .oneshot(send_subscription_request("/api/subscriptions", request))
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

        let new_subscription = NewSubscription::try_from(request.clone()).unwrap();

        // This mock storage returns an error which does not really makes
        // sense.
        let authentication_mock = MockAuthenticationStorage::new();
        let mut subscription_mock = MockSubscriptionStorage::new();

        subscription_mock
            .expect_create_subscription_and_store_token()
            .withf(move |subscription: &NewSubscription, _token: &str| {
                subscription == &new_subscription
            })
            .return_once(|_, _| {
                Err(SubscriptionError::Database {
                    context: "subscription context".to_string(),
                    source: sqlx::Error::RowNotFound.to_string(),
                })
            });
        subscription_mock
            .expect_get_subscription_by_email()
            .return_once(|_| Ok(None));

        let mut email_mock = MockEmailService::new();
        email_mock.expect_send_email().return_once(|_| Ok(()));

        let state = AppState {
            authentication: Arc::new(authentication_mock),
            subscription: Arc::new(subscription_mock),
            email: Arc::new(email_mock),
            base_url: ApplicationBaseUrl("http://127.0.0.1".to_string()),
            secret: Secret::new("secret".to_string()),
        };

        let app = subscription_route(state);

        let response = app
            .oneshot(send_subscription_request("/api/subscriptions", request))
            .await
            .expect("response");

        // Check the response status code.
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn subscription_should_return_error_if_invalid_data() {
        let username = Name().fake::<String>();
        let email = username.clone();

        let request = SubscriptionRequest { username, email };

        let authentication_mock = MockAuthenticationStorage::new();
        let subscription_mock = MockSubscriptionStorage::new();

        let email_mock = MockEmailService::new();

        let state = AppState {
            authentication: Arc::new(authentication_mock),
            subscription: Arc::new(subscription_mock),
            email: Arc::new(email_mock),
            base_url: ApplicationBaseUrl("http://127.0.0.1".to_string()),
            secret: Secret::new("secret".to_string()),
        };

        let app = subscription_route(state);

        let response = app
            .oneshot(send_subscription_request("/api/subscriptions", request))
            .await
            .expect("response");

        // Check the response status code.
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
