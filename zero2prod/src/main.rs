use clap::Parser;
use std::fmt;
use std::sync::Arc;

use zero2prod::email::Error as EmailError;
use zero2prod::email_client::EmailClient;
use zero2prod::listener::{listen_with_host_port, Error as ListenerError};
use zero2prod::opts::{Command, Error as OptsError, Opts};
use zero2prod::postgres::PostgresStorage;
use zero2prod::server;
use zero2prod::storage::Error as StorageError;
use zero2prod::telemetry;
use zero2prod_common::err_context::{ErrorContext, ErrorContextExt};
use zero2prod_common::settings::{Error as SettingsError, Settings};

#[derive(Debug)]
pub enum Error {
    Options {
        context: String,
        source: OptsError,
    },
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
    Email {
        context: String,
        source: EmailError,
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
            Error::Options { context, source } => {
                write!(fmt, "Options Error: {context} | {source}")
            }
            Error::Email { context, source } => {
                write!(fmt, "Email Error: {context} | {source}")
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

impl From<ErrorContext<String, OptsError>> for Error {
    fn from(err: ErrorContext<String, OptsError>) -> Self {
        Error::Options {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<String, EmailError>> for Error {
    fn from(err: ErrorContext<String, EmailError>) -> Self {
        Error::Email {
            context: err.0,
            source: err.1,
        }
    }
}

#[allow(clippy::result_large_err)]
#[tokio::main]
async fn main() -> Result<(), Error> {
    let subscriber =
        telemetry::get_subscriber("zero2prod".to_string(), "info".to_string(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let opts = Opts::parse();

    let cmd = opts.cmd.clone();

    let settings: Settings = opts
        .try_into()
        .context("Compiling Application Settings".to_string())?;

    match cmd {
        Command::Config => {
            println!("{}", serde_json::to_string_pretty(&settings).unwrap());
        }
        Command::Run => {
            let storage = Arc::new(
                PostgresStorage::new(settings.database)
                    .await
                    .context("Establishing a database connection".to_string())?,
            );

            let email = Arc::new(
                EmailClient::new(settings.email_client)
                    .await
                    .context("Establishing an email service connection".to_string())?,
            );

            let listener =
                listen_with_host_port(settings.network.host.as_str(), settings.network.port)
                    .context(format!(
                        "Could not create listener for {}:{}",
                        settings.network.host, settings.network.port
                    ))?;

            let state = server::State { storage, email };
            let server = server::run(listener, state);
            let server = tokio::spawn(server);
            if let Err(err) = server.await {
                eprintln!("Error: {err}");
            }
        }
    }
    Ok(())
}
