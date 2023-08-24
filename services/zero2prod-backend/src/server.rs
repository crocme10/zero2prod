/// This module holds the webserver specific details,
/// in our case all (most?) the axum related code.
use axum::{
    routing::{get, post, IntoMakeService, Router},
    Server,
};
use hyper::server::conn::AddrIncoming;
use secrecy::Secret;
use std::{fmt, net::TcpListener};
use std::{fmt::Display, sync::Arc};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::domain::ports::secondary::{AuthenticationStorage, EmailService, SubscriptionStorage};
use crate::routes::{
    authenticate::authenticate, health::health, login::login, logout::logout,
    newsletter::publish_newsletter, register::register,
    subscription_confirmation::subscriptions_confirmation, subscriptions::subscriptions,
};
use common::err_context::ErrorContext;

pub fn new(
    listener: TcpListener,
    authentication: Arc<dyn AuthenticationStorage + Send + Sync>,
    subscription: Arc<dyn SubscriptionStorage + Send + Sync>,
    email: Arc<dyn EmailService + Send + Sync>,
    base_url: String,
    _requests_per_sec: u64,
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
        .route("/api/v1/health", get(health))
        .route("/api/v1/subscriptions", post(subscriptions))
        .route(
            "/api/v1/subscriptions/confirmation",
            post(subscriptions_confirmation),
        )
        .route("/api/v1/newsletter", post(publish_newsletter))
        .route("/api/v1/login", post(login))
        .route("/api/v1/logout", get(logout))
        .route("/api/v1/register", post(register))
        .route("/api/v1/authenticate", get(authenticate));

    let cors = CorsLayer::permissive();

    let app = Router::new()
        .merge(router_no_session)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
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

#[derive(Debug)]
pub enum Error {
    Server {
        context: String,
        source: hyper::Error,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Server { context, source } => {
                write!(fmt, "Server: {context} | {source}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<ErrorContext<String, hyper::Error>> for Error {
    fn from(err: ErrorContext<String, hyper::Error>) -> Self {
        Error::Server {
            context: err.0,
            source: err.1,
        }
    }
}
