/// This module holds the webserver specific details,
/// in our case all (most?) the axum related code.
mod error;
pub use self::error::Error;

use axum::{
    error_handling::HandleErrorLayer,
    http::{header, HeaderValue, Method, StatusCode},
    routing::{get, get_service, post, IntoMakeService, Router},
    BoxError, Server,
};
use hyper::server::conn::AddrIncoming;
use secrecy::Secret;
use std::path::PathBuf;
use std::time::Duration;
use std::{fmt, net::TcpListener};
use std::{fmt::Display, sync::Arc};
use tower::ServiceBuilder;
use tower::{buffer::BufferLayer, limit::RateLimitLayer};
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use crate::domain::ports::secondary::{AuthenticationStorage, EmailService, SubscriptionStorage};
use crate::routes::{
    authenticate::authenticate, health::health, login::login, logout::logout,
    newsletter::publish_newsletter, register::register,
    subscription_confirmation::subscriptions_confirmation, subscriptions::subscriptions,
};

pub fn new(
    listener: TcpListener,
    authentication: Arc<dyn AuthenticationStorage + Send + Sync>,
    subscription: Arc<dyn SubscriptionStorage + Send + Sync>,
    email: Arc<dyn EmailService + Send + Sync>,
    base_url: String,
    static_dir: PathBuf,
    secret: Secret<String>,
) -> AppServer {
    // Build app state
    let app_state = AppState {
        authentication,
        subscription,
        email,
        base_url: ApplicationBaseUrl(base_url),
        secret,
    };

    // Routes that need to not have a session applied
    let router_no_session = Router::new()
        .route("/health", get(health))
        .route("/api/subscriptions", post(subscriptions))
        .route(
            "/api/subscriptions/confirmation",
            post(subscriptions_confirmation),
        )
        .route("/api/newsletter", post(publish_newsletter))
        .route("/api/v1/login", post(login))
        .route("/api/v1/logout", get(logout))
        .route("/api/v1/register", post(register))
        .route("/api/v1/authenticate", get(authenticate));

    let cors = CorsLayer::new()
        .allow_origin("http://127.0.0.1:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE]);
    // let cors = CorsLayer::permissive();

    let not_found_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("pages").join("404.html");
    let serve_dir = ServeDir::new(&static_dir).not_found_service(ServeFile::new(&not_found_path));

    // Create a router that will contain and match all routes for the application
    // and a fallback service that will serve the static directory
    tracing::info!("Serving static: {}", static_dir.display());
    let app = Router::new()
        .merge(router_no_session)
        .fallback_service(get_service(serve_dir).handle_error(|_| async {
            (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
        }))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_timeout_error))
                .layer(BufferLayer::new(1024))
                .layer(RateLimitLayer::new(5, Duration::from_secs(1))),
        )
        .with_state(app_state);

    // Start the axum server and set up to use supplied listener
    axum::Server::from_tcp(listener)
        .expect("failed to create server from listener")
        .serve(app.into_make_service())
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

pub type AppServer = Server<AddrIncoming, IntoMakeService<Router>>;

#[derive(Clone)]
pub struct ApplicationBaseUrl(pub String);

impl Display for ApplicationBaseUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// TODO Investigate:
// impl FromRef<AppState> for PgPool {
//
async fn handle_timeout_error(err: BoxError) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Unhandled internal error: {}", err),
    )
}
