use clap::Parser;
use std::fmt;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::signal;

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

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    let conn = pool.acquire()
        .await
        .map_err(|err| DatabaseError::DBConnection {
            context: "couldnot establish connection".to_string(),
            source: err
        }).context("Could not acquire db connection".to_string())?;

    let state = Arc::new(Mutex::new(server::State { exec: conn }));
    let server = server::run(listener, state, rx);
    let server = tokio::spawn(server);
    if let Err(err) = server.await {
        eprintln!("Error: {err}");
    }

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => { tx.send(()).expect("sig ctrlc") },
        _ = terminate => { tx.send(()).expect("sig terminate") },
    };

    Ok(())
}
