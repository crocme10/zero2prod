use axum::{
    extract::{rejection::JsonRejection, Json, State},
    routing::{get, post, Router},
    Server,
};
use axum_extra::extract::WithRejection;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::future::Future;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use tracing::info;

use crate::error::ApiError;
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

//pub async fn run<S>(opts: &Opts) -> Result<impl Future<Output = hyper::Result<()>>, Error> {
pub async fn run(opts: Opts) -> Result<(), Error> {
    let settings: Settings = opts.try_into().map_err(|err| Error::Configuration {
        context: "REST Server: Could not get server settings".to_string(),
        source: err,
    })?;

    let app_state = AppState {};

    let app = Router::new()
        .route("/health", get(health_endpoint))
        .with_state(app_state);

    let host = settings.network.host;
    let port = settings.network.port;
    let addr = (host.as_str(), port);
    let addr = addr
        .to_socket_addrs()
        .map_err(|err| Error::AddressDefinition {
            context: format!("REST Server: Could not resolve address  {host}:{port}"),
            source: err,
        })?
        .next()
        .ok_or_else(|| Error::AddressResolution {
            context: format!("REST Server: Could not resolve address  {host}:{port}",),
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

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        match rejection {
            JsonRejection::JsonDataError(_) => {
                tracing::error!("Invalid data");
                ApiError::new_bad_request(String::from("Invalid data"))
            }
            JsonRejection::MissingJsonContentType(_) => {
                tracing::error!("Missing JSON Content-Type");
                ApiError::new_bad_request(String::from("Missing JSON Content-Type"))
            }
            JsonRejection::JsonSyntaxError(_) => {
                tracing::error!("Invalid JSON Syntax");
                ApiError::new_bad_request(String::from("Invalid JSON Syntax"))
            }
            _ => {
                tracing::error!("Invalid JSON");
                ApiError::new_bad_request(String::from("Invalid JSON"))
            }
        }
    }
}

/// GET handler for health requests by an application platform
///
/// Intended for use in environments such as Amazon ECS or Kubernetes which want
/// to validate that the HTTP service is available for traffic, by returning a
/// 200 OK response with any content.
#[allow(clippy::unused_async)]
async fn health_endpoint() -> Json<NatterRestHealthResp> {
    let resp = NatterRestHealthResp {
        status: "OK".to_string(),
    };
    Json(resp)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatterRestHealthResp {
    pub status: String,
}
