/// This module holds the application definition, and
/// the creation of an application from settings.
///
/// Example:
///
///    let settings = Settings { ... }
///    let app = Application::build(settings).await?;
///    app.run_until_stopped().await?;
use crate::email_service::{EmailService, Error as EmailError};
use crate::email_service_impl::EmailClient;
use crate::listener::{listen_with_host_port, Error as ListenerError};
use crate::postgres::PostgresStorage;
use crate::server;
use crate::storage::{Error as StorageError, Storage};
use common::err_context::{ErrorContext, ErrorContextExt};
use common::settings::{ApplicationSettings, DatabaseSettings, EmailClientSettings, Settings};
use std::fmt;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;

pub struct Application {
    port: u16,
    server: server::AppServer,
}

impl Application {
    pub fn builder() -> ApplicationBuilder {
        ApplicationBuilder::default()
    }
}

#[derive(Default)]
pub struct ApplicationBuilder {
    pub storage: Option<Arc<dyn Storage + Send + Sync>>,
    pub email: Option<Arc<dyn EmailService + Send + Sync>>,
    pub listener: Option<TcpListener>,
    pub url: Option<String>,
    pub static_dir: Option<PathBuf>,
}

impl ApplicationBuilder {
    pub async fn new(settings: Settings) -> Result<Self, Error> {
        let Settings {
            application,
            database,
            email_client,
            mode: _,
        } = settings;
        let builder = Self::default()
            .storage(database)
            .await?
            .email(email_client)
            .await?
            .listener(application.clone())?
            .url(application.base_url)
            .static_dir(application.static_dir)?;

        Ok(builder)
    }

    pub async fn storage(mut self, settings: DatabaseSettings) -> Result<Self, Error> {
        let storage = Arc::new(
            PostgresStorage::new(settings)
                .await
                .context("Establishing a database connection".to_string())?,
        );
        self.storage = Some(storage);
        Ok(self)
    }

    pub async fn email(mut self, settings: EmailClientSettings) -> Result<Self, Error> {
        let email = Arc::new(
            EmailClient::new(settings)
                .await
                .context("Establishing an email service connection".to_string())?,
        );
        self.email = Some(email);
        Ok(self)
    }

    pub fn listener(mut self, settings: ApplicationSettings) -> Result<Self, Error> {
        let listener =
            listen_with_host_port(settings.host.as_str(), settings.port).context(format!(
                "Could not create listener for {}:{}",
                settings.host, settings.port
            ))?;
        self.listener = Some(listener);
        Ok(self)
    }

    pub fn url(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }

    pub fn static_dir(mut self, static_dir: String) -> Result<Self, Error> {
        let path = PathBuf::from(&static_dir);
        let path = if path.is_absolute() {
            path
        } else {
            let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            root.push(&path);
            root
        };
        let path = path
            .canonicalize()
            .context("Could not canonicalize static dir".to_string())?;
        self.static_dir = Some(path);
        Ok(self)
    }

    pub fn build(self) -> Application {
        let ApplicationBuilder {
            storage,
            email,
            listener,
            url,
            static_dir,
        } = self;
        let listener = listener.expect("listener");
        let port = listener.local_addr().expect("listener local addr").port();
        let server = server::new(
            listener,
            storage.expect("storage"),
            email.expect("email"),
            url.expect("url"),
            static_dir.expect("static dir"),
        );
        Application { port, server }
    }
}

impl Application {
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
    Path {
        context: String,
        source: std::io::Error,
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
            Error::Path { context, source } => {
                write!(fmt, "IO Error: {context} | {source}")
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

impl From<ErrorContext<String, std::io::Error>> for Error {
    fn from(err: ErrorContext<String, std::io::Error>) -> Self {
        Error::Path {
            context: err.0,
            source: err.1,
        }
    }
}
