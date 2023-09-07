pub mod context;
pub mod cookies;
mod middleware;
pub mod routes;

use axum::{
    http::{header, HeaderValue, Method},
    middleware::{from_fn_with_state, map_response},
    routing::Router,
};
use axum_server::{accept::DefaultAcceptor, Server};
use secrecy::Secret;
use std::{fmt, net::TcpListener, sync::Arc};
use tower_cookies::CookieManagerLayer;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use self::middleware::resolve_context::resolve_context;
use self::middleware::response_map::error;
use crate::domain::ports::secondary::{AuthenticationStorage, EmailService, SubscriptionStorage};
use crate::utils::tracing::make_span;

pub fn new(listener: TcpListener, state: AppState) -> (Router, Server<DefaultAcceptor>) {
    // FIXME Hardcoded origin
    let cors = CorsLayer::new()
        .allow_origin("http://127.0.0.1:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE]);

    let router = Router::new()
        .nest("/api/v1", routes::routes(state.clone()))
        .layer(map_response(error))
        .layer(from_fn_with_state(state.clone(), resolve_context))
        .layer(CookieManagerLayer::new())
        .layer(cors)
        .layer(TraceLayer::new_for_http().make_span_with(make_span));

    let server = axum_server::from_tcp(listener);

    (router, server)
}

pub type DynAuthentication = Arc<dyn AuthenticationStorage + Send + Sync>;
pub type DynSubscription = Arc<dyn SubscriptionStorage + Send + Sync>;
pub type DynEmail = Arc<dyn EmailService + Send + Sync>;

#[derive(Clone)]
pub struct AppState {
    pub subscription: DynSubscription,
    pub authentication: DynAuthentication,
    pub email: DynEmail,
    pub base_url: ApplicationBaseUrl,
    pub secret: Secret<String>,
}

pub type AppServer = Server<DefaultAcceptor>;

#[derive(Clone)]
pub struct ApplicationBaseUrl(pub String);

impl fmt::Display for ApplicationBaseUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
