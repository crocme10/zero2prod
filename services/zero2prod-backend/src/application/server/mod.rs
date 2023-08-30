pub mod context;
pub mod cookies;
mod middleware;
pub mod routes;

use axum::{
    error_handling::HandleErrorLayer,
    http::{header, HeaderValue, Method, StatusCode},
    middleware::{from_fn_with_state, map_response},
    routing::Router,
    BoxError,
};
use axum_server::tls_rustls::{RustlsAcceptor, RustlsConfig};
use axum_server::Server;
use secrecy::Secret;
use std::path::PathBuf;
use std::time::Duration;
use std::{fmt, net::TcpListener};
use std::{fmt::Display, sync::Arc};
use tower::ServiceBuilder;
use tower::{buffer::BufferLayer, limit::RateLimitLayer};
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use self::middleware::resolve_context::resolve_context;
use self::middleware::response_map::error;
use crate::domain::ports::secondary::{AuthenticationStorage, EmailService, SubscriptionStorage};

pub fn new(
    listener: TcpListener,
    state: AppState,
    static_dir: PathBuf,
    tls: RustlsConfig,
) -> (Router, Server<RustlsAcceptor>) {
    // FIXME Hardcoded origin
    let cors = CorsLayer::new()
        .allow_origin("http://127.0.0.1:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE]);

    tracing::info!("Serving static directory: {}", static_dir.display());

    let router = Router::new()
        .merge(routes::routes(state.clone()))
        .fallback_service(routes::static_dir::static_dir(static_dir))
        .layer(map_response(error))
        .layer(from_fn_with_state(state.clone(), resolve_context))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(CookieManagerLayer::new())
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_timeout_error))
                .layer(BufferLayer::new(1024))
                .layer(RateLimitLayer::new(5, Duration::from_secs(1))),
        );

    let server = axum_server::from_tcp_rustls(listener, tls);

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

pub type AppServer = Server<RustlsAcceptor>;

#[derive(Clone)]
pub struct ApplicationBaseUrl(pub String);

impl Display for ApplicationBaseUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

async fn handle_timeout_error(err: BoxError) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Unhandled internal error: {}", err),
    )
}
