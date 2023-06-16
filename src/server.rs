use axum::{
    routing::{get, post, Router},
    Server,
};
use std::sync::Arc;
use std::{fmt, net::TcpListener};

use crate::err_context::{ErrorContext, ErrorContextExt};
use crate::routes::{health::health, subscriptions::subscriptions};
use crate::storage::Storage;

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

pub async fn run(listener: TcpListener, state: State) -> Result<(), Error> {
    let app = Router::new()
        .route("/health", get(health))
        .route("/subscriptions", post(subscriptions))
        .with_state(state);

    Server::from_tcp(listener)
        .context("Could not create server from TCP Listener".to_string())?
        .serve(app.into_make_service())
        .await
        .map_err(|err| Error::Server {
            context: "REST Server".to_string(),
            source: err,
        })
}

#[derive(Clone)]
pub struct State {
    pub storage: Arc<dyn Storage + Send + Sync>,
}
