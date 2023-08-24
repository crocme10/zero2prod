/// This module holds the application definition, and
/// the creation of an application from settings.
///
/// Example:
///
///    let settings = Settings { ... }
///    let app = Application::build(settings).await?;
///    app.run_until_stopped().await?;
use common::err_context::{ErrorContext, ErrorContextExt};
use common::settings::{ApplicationSettings, DatabaseSettings, EmailClientSettings, Settings};
use secrecy::Secret;
use std::fmt;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;

use crate::domain::ports::secondary::{
    AuthenticationError, AuthenticationStorage, EmailError, EmailService, SubscriptionError,
    SubscriptionStorage,
};
use crate::listener::{listen_with_host_port, Error as ListenerError};
use crate::server;
use crate::services::email::EmailClient;
use crate::services::postgres::{Error as PostgresError, PostgresStorage};

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
    pub authentication: Option<Arc<dyn AuthenticationStorage + Send + Sync>>,
    pub subscription: Option<Arc<dyn SubscriptionStorage + Send + Sync>>,
    pub email: Option<Arc<dyn EmailService + Send + Sync>>,
    pub listener: Option<TcpListener>,
    pub url: Option<String>,
    pub static_dir: Option<PathBuf>,
    pub requests_per_sec: Option<u64>,
    pub secret: Option<Secret<String>>,
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
            .authentication(database.clone())
            .await?
            .subscription(database)
            .await?
            .email(email_client)
            .await?
            .listener(application.clone())?
            .url(application.base_url)
            .static_dir(application.static_dir)?
            .requests_per_sec(application.requests_per_sec)
            .secret("Secret".to_string());

        Ok(builder)
    }

    pub async fn authentication(mut self, settings: DatabaseSettings) -> Result<Self, Error> {
        let storage = Arc::new(
            PostgresStorage::new(settings)
                .await
                .context("Establishing a database connection".to_string())?,
        );
        self.authentication = Some(storage);
        Ok(self)
    }

    pub async fn subscription(mut self, settings: DatabaseSettings) -> Result<Self, Error> {
        let storage = Arc::new(
            PostgresStorage::new(settings)
                .await
                .context("Establishing a database connection".to_string())?,
        );
        self.subscription = Some(storage);
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

    pub fn requests_per_sec(mut self, requests_per_sec: u64) -> Self {
        self.requests_per_sec = Some(requests_per_sec);
        self
    }

    pub fn secret(mut self, secret: String) -> Self {
        self.secret = Some(Secret::new(secret));
        self
    }

    pub fn build(self) -> Application {
        let ApplicationBuilder {
            authentication,
            subscription,
            email,
            listener,
            url,
            static_dir,
            requests_per_sec,
            secret,
        } = self;
        let listener = listener.expect("listener");
        let port = listener.local_addr().expect("listener local addr").port();
        let server = server::new(
            listener,
            authentication.expect("authentication"),
            subscription.expect("subscription"),
            email.expect("email"),
            url.expect("url"),
            static_dir.expect("static dir"),
            requests_per_sec.expect("requests per sec"),
            secret.expect("secret"),
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
    Postgres {
        context: String,
        source: PostgresError,
    },
    Authentication {
        context: String,
        source: AuthenticationError,
    },
    Subscription {
        context: String,
        source: SubscriptionError,
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
            Error::Postgres { context, source } => {
                write!(fmt, "Storage Error: {context} | {source}")
            }
            Error::Authentication { context, source } => {
                write!(fmt, "Authentication Error: {context} | {source}")
            }
            Error::Subscription { context, source } => {
                write!(fmt, "Subscription Error: {context} | {source}")
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

impl From<ErrorContext<String, AuthenticationError>> for Error {
    fn from(err: ErrorContext<String, AuthenticationError>) -> Self {
        Error::Authentication {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<String, PostgresError>> for Error {
    fn from(err: ErrorContext<String, PostgresError>) -> Self {
        Error::Postgres {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<String, SubscriptionError>> for Error {
    fn from(err: ErrorContext<String, SubscriptionError>) -> Self {
        Error::Subscription {
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

impl From<ErrorContext<String, ListenerError>> for Error {
    fn from(err: ErrorContext<String, ListenerError>) -> Self {
        Error::Listener {
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
