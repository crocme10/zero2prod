use clap::Parser;
use std::fmt;

use zero2prod::database::{connect_with_conn_str, Error as DatabaseError};
use zero2prod::err_context::{ErrorContext, ErrorContextExt};
use zero2prod::listener::{listen_with_host_port, Error as ListenerError};
use zero2prod::server;
use zero2prod::settings::{Error as SettingsError, Opts, Settings};

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
    // Server {
    //     context: String,
    //     source: hyper::Error,
    // },
    Database {
        context: String,
        source: DatabaseError,
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
            // Error::Server { context, source } => {
            //     write!(fmt, "Server: {context} | {source}")
            // }
            Error::Database { context, source } => {
                write!(fmt, "Database Error: {context} | {source}")
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

impl From<ErrorContext<String, DatabaseError>> for Error {
    fn from(err: ErrorContext<String, DatabaseError>) -> Self {
        Error::Database {
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
    tracing_subscriber::fmt::init();

    let opts = Opts::parse();

    let settings: Settings = opts
        .try_into()
        .context("Compiling Application Settings".to_string())?;

    let conn_str = settings.database.connection_string();

    tracing::info!("Establishing database connection with {}", conn_str);

    let pool = connect_with_conn_str(&conn_str, settings.database.connection_timeout)
        .await
        .context(format!(
            "Establishing a database connection with {conn_str}"
        ))?;

    let listener = listen_with_host_port(settings.network.host.as_str(), settings.network.port)
        .context(format!(
            "Could not create listener for {}:{}",
            settings.network.host, settings.network.port
        ))?;

    let server = server::run(listener, pool);
    let server = tokio::spawn(server);
    if let Err(err) = server.await {
        eprintln!("Error: {err}");
    }

    Ok(())
}
