mod error;
mod listener;
pub mod opts;
pub mod server;

pub use self::error::Error;

use axum::routing::Router;
use common::err_context::ErrorContextExt;
use common::settings::{ApplicationSettings, DatabaseSettings, EmailClientSettings, Settings};
use secrecy::Secret;
use std::net::TcpListener;
use std::sync::Arc;

use self::listener::listen_with_host_port;
use crate::domain::ports::secondary::{AuthenticationStorage, EmailService, SubscriptionStorage};
use crate::services::email::EmailClient;
use crate::services::postgres::PostgresStorage;

pub struct Application {
    http: u16,
    app: Router,
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
    pub http: Option<u16>,
    pub url: Option<String>,
    pub secret: Option<Secret<String>>,
}

impl ApplicationBuilder {
    pub async fn new(settings: Settings) -> Result<Self, Error> {
        let Settings {
            application,
            database,
            email_client,
            tracing: _,
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
            .http(application.http)
            .url(application.base_url)
            .secret("Secret".to_string());

        Ok(builder)
    }

    pub async fn authentication(mut self, settings: DatabaseSettings) -> Result<Self, Error> {
        let storage = Arc::new(
            PostgresStorage::new(settings)
                .await
                .context("Establishing a database connection")?,
        );
        self.authentication = Some(storage);
        Ok(self)
    }

    pub async fn subscription(mut self, settings: DatabaseSettings) -> Result<Self, Error> {
        let storage = Arc::new(
            PostgresStorage::new(settings)
                .await
                .context("Establishing a database connection")?,
        );
        self.subscription = Some(storage);
        Ok(self)
    }

    pub async fn email(mut self, settings: EmailClientSettings) -> Result<Self, Error> {
        let email = Arc::new(
            EmailClient::new(settings)
                .await
                .context("Establishing an email service connection")?,
        );
        self.email = Some(email);
        Ok(self)
    }

    pub fn listener(mut self, settings: ApplicationSettings) -> Result<Self, Error> {
        let listener =
            listen_with_host_port(settings.host.as_str(), settings.http).context(format!(
                "Could not create listener for {}:{}",
                settings.host, settings.http
            ))?;
        tracing::info!("Created listener on port: {}", settings.http);
        self.listener = Some(listener);
        Ok(self)
    }

    pub fn http(mut self, port: u16) -> Self {
        self.http = Some(port);
        self
    }

    pub fn url(mut self, url: String) -> Self {
        self.url = Some(url);
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
            http,
            url,
            secret,
        } = self;
        let listener = listener.expect("listener");
        let state = server::AppState {
            authentication: authentication.expect("authentication"),
            subscription: subscription.expect("subscription"),
            email: email.expect("email"),
            base_url: server::ApplicationBaseUrl(url.expect("url")),
            secret: secret.expect("secret"),
        };

        let (app, server) = server::new(listener, state);

        Application {
            http: http.expect("http"),
            app,
            server,
        }
    }
}

impl Application {
    pub fn port(&self) -> u16 {
        self.http
    }

    pub async fn run_until_stopped(self) -> Result<(), Error> {
        self.server
            .serve(self.app.into_make_service())
            .await
            .context("server execution error")?;
        Ok(())
    }
}
