mod error;
mod listener;
pub mod server;
pub mod opts;

pub use self::error::Error;

use common::err_context::ErrorContextExt;
use common::settings::{ApplicationSettings, DatabaseSettings, EmailClientSettings, Settings};
use secrecy::Secret;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;

use self::listener::listen_with_host_port;
use crate::domain::ports::secondary::{AuthenticationStorage, EmailService, SubscriptionStorage};
use crate::services::email::EmailClient;
use crate::services::postgres::PostgresStorage;

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
    pub https_listener: Option<TcpListener>,
    pub http_listener: Option<TcpListener>,
    pub url: Option<String>,
    pub static_dir: Option<PathBuf>,
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
            .https_listener(application.clone())?
            .http_listener(application.clone())?
            .url(application.base_url)
            .static_dir(application.static_dir)?
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

    pub fn https_listener(mut self, settings: ApplicationSettings) -> Result<Self, Error> {
        let listener =
            listen_with_host_port(settings.host.as_str(), settings.https).context(format!(
                "Could not create listener for {}:{}",
                settings.host, settings.https
            ))?;
        self.https_listener = Some(listener);
        Ok(self)
    }

    pub fn http_listener(mut self, settings: ApplicationSettings) -> Result<Self, Error> {
        let listener =
            listen_with_host_port(settings.host.as_str(), settings.http).context(format!(
                "Could not create listener for {}:{}",
                settings.host, settings.http
            ))?;
        self.http_listener = Some(listener);
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

    pub fn secret(mut self, secret: String) -> Self {
        self.secret = Some(Secret::new(secret));
        self
    }

    pub fn build(self) -> Application {
        let ApplicationBuilder {
            authentication,
            subscription,
            email,
            http_listener,
            https_listener,
            url,
            static_dir,
            secret,
        } = self;
        let http_listener = http_listener.expect("listener");
        let http = http_listener.local_addr().expect("listener local addr").port();
        let https_listener = https_listener.expect("listener");
        let https = https_listener.local_addr().expect("listener local addr").port();
        let server = server::new(
            https_listener,
            authentication.expect("authentication"),
            subscription.expect("subscription"),
            email.expect("email"),
            url.expect("url"),
            static_dir.expect("static dir"),
            secret.expect("secret"),
        );
        Application { port: https, server }
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
