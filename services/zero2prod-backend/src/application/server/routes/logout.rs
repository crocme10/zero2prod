use axum::extract::Json;
use axum::response::IntoResponse;
use tower_cookies::Cookies;
use uuid::Uuid;

use super::Error;
use crate::application::server::cookies;

/// GET handler for user logout
#[allow(clippy::unused_async)]
#[tracing::instrument(
    name = "User Logout"
    fields(
        request_id = %Uuid::new_v4(),
    )
)]
pub async fn logout(cookies: Cookies) -> Result<impl IntoResponse, Error> {
    cookies::remove_token_cookie(&cookies);
    Ok::<_, Error>(Json(serde_json::json!({
        "status": "success"
    })))
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{header, Request, StatusCode},
        middleware::{from_fn_with_state, map_response},
        routing::{get, Router},
    };
    use hyper::header::SET_COOKIE;
    use mockall::predicate::*;
    use secrecy::Secret;
    use std::sync::Arc;
    use tower::ServiceExt;
    use tower_cookies::{cookie::time::Duration, Cookie, CookieManagerLayer};

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

    fn logout_route(state: AppState) -> Router {
        Router::new()
            .route("/api/logout", get(logout))
            .layer(map_response(error))
            .layer(from_fn_with_state(state.clone(), resolve_context))
            .layer(CookieManagerLayer::new())
            .with_state(state)
    }

    fn send_logout_request(uri: &str) -> Request<Body> {
        Request::builder()
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::ACCEPT, "application/json")
            .method("GET")
            .body(Body::empty())
            .unwrap()
    }

    #[tokio::test]
    async fn logout_should_set_a_cookie() {
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

        let app = logout_route(state);

        let response = app
            .oneshot(send_logout_request("/api/logout"))
            .await
            .expect("response");

        // Check the response status code.
        assert_eq!(response.status(), StatusCode::OK);

        let cookie = response
            .headers()
            .get(SET_COOKIE)
            .unwrap()
            .to_str()
            .unwrap();
        let cookie = Cookie::parse(cookie).unwrap();
        assert_eq!(cookie.max_age(), Some(Duration::ZERO))
    }
}
