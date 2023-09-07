mod error;

pub use self::error::Error;
use crate::application::server::context::Context;
use crate::application::server::cookies::JWT;
use crate::application::server::routes::Error as RoutesError;
use crate::application::server::AppState;
use crate::authentication::jwt::Authenticator;

use axum::extract::State;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use common::err_context::ErrorContextExt;
use std::fmt;
use tower_cookies::{Cookie, Cookies};

#[tracing::instrument(
    name = "Context Resolution"
    skip(state, cookies, req, next)
)]
pub async fn resolve_context<B: fmt::Debug>(
    state: State<AppState>,
    cookies: Cookies,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, RoutesError> {
    println!("resolve context");
    let context = resolve(&cookies, state).await;

    if context.is_err() && !matches!(context, Err(Error::TokenNotFound)) {
        cookies.remove(Cookie::named(JWT))
    }
    
    let context = context.context("Could not extract context")?;

    req.extensions_mut().insert(context);

    Ok(next.run(req).await)
}

pub async fn resolve(cookies: &Cookies, State(state): State<AppState>) -> Result<Context, Error> {
    let token = cookies.get(JWT).map(|cookie| cookie.value().to_string());

    let token = token.ok_or(Error::TokenNotFound)?;

    println!("found token");

    let authenticator = Authenticator {
        storage: state.authentication.clone(),
        secret: state.secret.clone(),
    };

    let id = authenticator
        .validate_token(&token)
        .await
        .context("Could not validate token")?;

    println!("inserting id into context {}", id);

    Context::new(Some(id)).map_err(|_| Error::InvalidUserId {
        context: "invalid user id".to_string(),
    })
}
