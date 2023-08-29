mod error;

pub use self::error::Error;
use crate::application::server::context::Context;
use crate::application::server::cookies::JWT;
use crate::application::server::AppState;
use crate::application::server::routes::Error as RoutesError;
use crate::authentication::jwt::Authenticator;

use axum::extract::State;
use axum::http::{header, Request};
use axum::middleware::Next;
use axum::response::Response;
use common::err_context::ErrorContextExt;
use std::fmt;
use tower_cookies::{Cookie, Cookies};

// #[allow(dead_code)] // For now, until we have the rpc.
// pub async fn mw_ctx_require<B>(
// 	ctx: Result<Ctx>,
// 	req: Request<B>,
// 	next: Next<B>,
// ) -> Result<Response> {
// 	debug!("{:<12} - mw_ctx_require - {ctx:?}", "MIDDLEWARE");
//
// 	ctx?;
//
// 	Ok(next.run(req).await)
// }

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
    let context = resolve(&cookies, state, &req).await;

    if context.is_err() && !matches!(context, Err(Error::TokenNotFound)) {
        cookies.remove(Cookie::named(JWT))
    }

    req.extensions_mut().insert(context);

    Ok(next.run(req).await)
}

pub async fn resolve<B: fmt::Debug>(
    cookies: &Cookies,
    State(state): State<AppState>,
    req: &Request<B>,
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
