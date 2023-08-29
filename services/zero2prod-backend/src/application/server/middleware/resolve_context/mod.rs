mod error;

pub use self::error::Error;
use crate::application::server::AppState;
use crate::authentication::jwt::Authenticator;
use common::err_context::ErrorContextExt;

use crate::application::server::context::Context;
use axum::extract::State;
use axum::http::{header, Request};
use std::fmt;
use tower_cookies::Cookies;

#[allow(clippy::unused_async)]
#[tracing::instrument(
    name = "Context Resolution"
    skip(state, cookies)
)]
pub async fn resolve_context<B: fmt::Debug>(
    cookies: &Cookies,
    State(state): State<AppState>,
    req: Request<B>,
) -> Result<Context, Error> {
    let token = cookies
        .get("jwt") // FIXME hardcoded
        .map(|cookie| cookie.value().to_string())
        .or_else(|| {
            req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth_header| auth_header.to_str().ok())
                .and_then(|auth_value| {
                    auth_value
                        .strip_prefix("Bearer ")
                        .map(|token| token.to_owned())
                })
        });

    let token = token.ok_or_else(|| Error::TokenNotFound)?;

    let authenticator = Authenticator {
        storage: state.authentication.clone(),
        secret: state.secret.clone(),
    };

    let id = authenticator
        .validate_token(&token)
        .await
        .context("Could not validate token")?;

    Context::new(Some(id)).map_err(|_| Error::InvalidUserId {
        context: "invalid user id".to_string(),
    })
}
