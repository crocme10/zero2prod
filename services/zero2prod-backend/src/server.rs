/// This module holds the webserver specific details,
/// in our case all (most?) the axum related code.
use axum::{
    routing::{get, post, IntoMakeService, Router},
    Server,
    http::{Response, StatusCode},
    body::{boxed, Body},
};
use hyper::server::conn::AddrIncoming;
use std::{fmt, net::TcpListener};
use std::{fmt::Display, sync::Arc};
use tower_http::trace::TraceLayer;
use std::path::PathBuf;
use tower_http::services::ServeDir;
use tower::util::ServiceExt;
use tokio::fs;

use crate::email_service::EmailService;
use crate::routes::{
    health::health, subscription_confirmation::subscriptions_confirmation,
    subscriptions::subscriptions,
};
use crate::storage::Storage;
use zero2prod_common::err_context::ErrorContext;

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

pub fn new(
    listener: TcpListener,
    storage: Arc<dyn Storage + Send + Sync>,
    email: Arc<dyn EmailService + Send + Sync>,
    base_url: String,
    static_dir: PathBuf,
) -> AppServer {
    // Build app state
    let app_state = AppState {
        storage,
        email,
        base_url: ApplicationBaseUrl(base_url),
    };

    // Routes that need to not have a session applied
    let router_no_session = Router::new()
        .route("/health", get(health))
        .route("/subscriptions", post(subscriptions))
        .route(
            "/subscriptions/confirmation",
            post(subscriptions_confirmation),
        );

    // Create a router that will contain and match all routes for the application
    // and a fallback service that will serve the static directory
    tracing::info!("Serving static: {}", static_dir.display());
    let app = Router::new()
        .merge(router_no_session)
        .fallback_service(get(|req| async move {
            match ServeDir::new(&static_dir).oneshot(req).await {
                Ok(res) => {
                    let status = res.status();
                    match status {
                        StatusCode::NOT_FOUND => {
                            let index_path = PathBuf::from(&static_dir).join("index.html");
                            let index_content = match fs::read_to_string(index_path).await {
                                Err(_) => {
                                    return Response::builder()
                                        .status(StatusCode::NOT_FOUND)
                                        .body(boxed(Body::from("index file not found")))
                                        .unwrap()
                                }
                                Ok(index_content) => index_content,
                            };

                            Response::builder()
                                .status(StatusCode::OK)
                                .body(boxed(Body::from(index_content)))
                                .unwrap()
                        }
                        _ => res.map(boxed),
                    }
                }
                Err(err) => Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(boxed(Body::from(format!("error: {err}"))))
                    .expect("error response"),
            }
        }))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // Start the axum server and set up to use supplied listener
    axum::Server::from_tcp(listener)
        .expect("failed to create server from listener")
        .serve(app.into_make_service())
}

pub type DynStorage = Arc<dyn Storage + Send + Sync>;
pub type DynEmail = Arc<dyn EmailService + Send + Sync>;

#[derive(Clone)]
pub struct AppState {
    pub storage: DynStorage,
    pub email: DynEmail,
    pub base_url: ApplicationBaseUrl,
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
