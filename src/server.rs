use axum::{
    routing::{get, post, Router},
    Server,
};
use std::fmt;
use std::net::ToSocketAddrs;

use crate::err_context::ErrorContext;
use crate::routes::{health::health, subscriptions::subscriptions};
use crate::settings::{Error as SettingsError, Opts, Settings};

#[derive(Debug)]
pub enum Error {
    AddressResolution {
        context: String,
    },
    AddressDefinition {
        context: String,
        source: std::io::Error,
    },
    Configuration {
        context: String,
        source: SettingsError,
    },
    Server {
        context: String,
        source: hyper::Error,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Configuration { context, source } => {
                write!(
                    fmt,
                    "REST Server: Could not build configuration: {context} | {source}"
                )
            }
            Error::AddressResolution { context } => {
                write!(
                    fmt,
                    "REST Server: Could not resolve server address: {context}",
                )
            }
            Error::AddressDefinition { context, source } => {
                write!(fmt, "Could not build client request: {context} | {source}")
            }
            Error::Server { context, source } => {
                write!(fmt, "Server: {context} | {source}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<ErrorContext<String, SettingsError>> for Error {
    fn from(err: ErrorContext<String, SettingsError>) -> Self {
        Error::Configuration {
            context: err.0,
            source: err.1,
        }
    }
}

pub async fn run(settings: Settings) -> Result<(), Error> {
    let app_state = AppState {};

    let app = Router::new()
        .route("/health", get(health))
        .route("/subscriptions", post(subscriptions))
        .with_state(app_state);

    let addr = (settings.network.host.as_str(), settings.network.port);
    let addr = addr
        .to_socket_addrs()
        .map_err(|err| Error::AddressDefinition {
            context: format!(
                "REST Server: Could not resolve address  {}:{}",
                settings.network.host, settings.network.port
            ),
            source: err,
        })?
        .next()
        .ok_or_else(|| Error::AddressResolution {
            context: format!(
                "REST Server: Could not resolve address  {}:{}",
                settings.network.host, settings.network.port
            ),
        })?;

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|err| Error::Server {
            context: format!("REST Server"),
            source: err,
        })
}

pub async fn config(opts: Opts) -> Result<(), Error> {
    let settings: Settings = opts.try_into().map_err(|err| Error::Configuration {
        context: "REST Server: Could not get server settings".to_string(),
        source: err,
    })?;
    println!("{}", serde_json::to_string_pretty(&settings).unwrap());
    Ok(())
}

#[derive(Clone)]
pub struct AppState {}
