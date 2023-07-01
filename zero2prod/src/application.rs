/// This module holds the application definition, and
/// the creation of an application from settings.
///
/// Example:
///
///    let settings = Settings { ... }
///    let app = Application::build(settings).await?;
///    app.run_until_stopped().await?;
use crate::email_service::Error as EmailError;
use crate::email_service_impl::EmailClient;
use crate::listener::{listen_with_host_port, Error as ListenerError};
use crate::postgres::PostgresStorage;
use crate::server;
use crate::storage::Error as StorageError;
use zero2prod_common::err_context::{ErrorContext, ErrorContextExt};
use zero2prod_common::settings::Settings;

use std::fmt;
use std::sync::Arc;

pub struct Application {
    port: u16,
    server: server::AppServer,
}

impl Application {
    pub async fn new(settings: Settings) -> Result<Self, Error> {
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

        let listener = listen_with_host_port(settings.network.host.as_str(), settings.network.port)
            .context(format!(
                "Could not create listener for {}:{}",
                settings.network.host, settings.network.port
            ))?;

        let port = listener.local_addr().unwrap().port();
        let server = server::new(listener, storage, email, settings.network.host);
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), Error> {
        self.server
            .await
            .context("server execution error".to_string())?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    Listener {
        context: String,
        source: ListenerError,
    },
    Storage {
        context: String,
        source: StorageError,
    },
    Email {
        context: String,
        source: EmailError,
    },
    Server {
        context: String,
        source: hyper::Error,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Listener { context, source } => {
                write!(fmt, "Could not build TCP listener: {context} | {source}")
            }
            Error::Storage { context, source } => {
                write!(fmt, "Storage Error: {context} | {source}")
            }
            Error::Email { context, source } => {
                write!(fmt, "Email Error: {context} | {source}")
            }
            Error::Server { context, source } => {
                write!(fmt, "Application Server Error: {context} | {source}")
            }
        }
    }
}

impl std::error::Error for Error {}

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

impl From<ErrorContext<String, EmailError>> for Error {
    fn from(err: ErrorContext<String, EmailError>) -> Self {
        Error::Email {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<String, hyper::Error>> for Error {
    fn from(err: ErrorContext<String, hyper::Error>) -> Self {
        Error::Server {
            context: err.0,
            source: err.1,
        }
    }
}
