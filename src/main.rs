use clap::Parser;
use std::fmt;
use std::sync::Arc;

use zero2prod::err_context::{ErrorContext, ErrorContextExt};
use zero2prod::listener::{listen_with_host_port, Error as ListenerError};
use zero2prod::postgres::PostgresStorage;
use zero2prod::server;
use zero2prod::settings::{Error as SettingsError, Opts, Settings};
use zero2prod::storage::Error as StorageError;
use zero2prod::telemetry;

#[derive(Debug)]
pub enum Error {
    Listener {
        context: String,
        source: ListenerError,
    },
    Configuration {
        context: String,
        source: SettingsError,
    },
    Storage {
        context: String,
        source: StorageError,
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
            Error::Listener { context, source } => {
                write!(fmt, "Could not build TCP listener: {context} | {source}")
            }
            Error::Storage { context, source } => {
                write!(fmt, "Storage Error: {context} | {source}")
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

impl From<ErrorContext<String, StorageError>> for Error {
    fn from(err: ErrorContext<String, StorageError>) -> Self {
        Error::Storage {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<String, ListenerError>> for Error {
    fn from(err: ErrorContext<String, ListenerError>) -> Self {
        Error::Listener {
            context: err.0,
            source: err.1,
        }
    }
}

#[allow(clippy::result_large_err)]
#[tokio::main]
async fn main() -> Result<(), Error> {
    let subscriber = telemetry::get_subscriber("zero2prod".to_string(), "info".to_string(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let opts = Opts::parse();

    let settings: Settings = opts
        .try_into()
        .context("Compiling Application Settings".to_string())?;

    let storage = Arc::new(
        PostgresStorage::new(settings.database)
            .await
            .context("Establishing a database connection".to_string())?,
    );

    let listener = listen_with_host_port(settings.network.host.as_str(), settings.network.port)
        .context(format!(
            "Could not create listener for {}:{}",
            settings.network.host, settings.network.port
        ))?;

    let state = server::State { storage };
    let server = server::run(listener, state);
    let server = tokio::spawn(server);
    if let Err(err) = server.await {
        eprintln!("Error: {err}");
    }

    Ok(())
}
