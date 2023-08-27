mod error;
mod listener;
pub mod opts;
pub mod server;

pub use self::error::Error;

use axum::{
    extract::Host,
    handler::HandlerWithoutStateExt,
    http::{StatusCode, Uri},
    response::Redirect,
    BoxError, Server,
};
use common::err_context::ErrorContextExt;
use common::settings::{ApplicationSettings, DatabaseSettings, EmailClientSettings, Settings};
use secrecy::Secret;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;

use self::listener::listen_with_host_port;
use crate::domain::ports::secondary::{AuthenticationStorage, EmailService, SubscriptionStorage};
use crate::services::email::EmailClient;
use crate::services::postgres::PostgresStorage;

pub struct Application {
    http: u16,
    https: u16,
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
    pub https: Option<u16>,
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
            .listener(application.clone())?
            .http(application.http)
            .https(application.https)
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

    pub fn listener(mut self, settings: ApplicationSettings) -> Result<Self, Error> {
        let listener =
            listen_with_host_port(settings.host.as_str(), settings.https).context(format!(
                "Could not create listener for {}:{}",
                settings.host, settings.https
            ))?;
        self.listener = Some(listener);
        Ok(self)
    }

    pub fn http(mut self, port: u16) -> Self {
        self.http = Some(port);
        self
    }

    pub fn https(mut self, port: u16) -> Self {
        self.https = Some(port);
        self
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
            listener,
            http,
            https,
            url,
            static_dir,
            secret,
        } = self;
        let listener = listener.expect("listener");
        let server = server::new(
            listener,
            authentication.expect("authentication"),
            subscription.expect("subscription"),
            email.expect("email"),
            url.expect("url"),
            static_dir.expect("static dir"),
            secret.expect("secret"),
        );
        Application {
            http: http.expect("http"),
            https: https.expect("https"),
            server,
        }
    }
}

impl Application {
    pub fn port(&self) -> u16 {
        self.https
    }

    pub async fn run_until_stopped(self) -> Result<(), Error> {
        tokio::spawn(redirect_http_to_https(Ports {
            http: self.http,
            https: self.https,
        }));
        self.server
            .await
            .context("server execution error".to_string())?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct Ports {
    http: u16,
    https: u16,
}

async fn redirect_http_to_https(ports: Ports) {
    fn make_https(host: String, uri: Uri, ports: Ports) -> Result<Uri, BoxError> {
        let mut parts = uri.into_parts();

        parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".parse().unwrap());
        }

        let https_host = host.replace(&ports.http.to_string(), &ports.https.to_string());
        parts.authority = Some(https_host.parse()?);

        Ok(Uri::from_parts(parts)?)
    }

    let redirect = move |Host(host): Host, uri: Uri| async move {
        match make_https(host, uri, ports) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(error) => {
                tracing::warn!(%error, "failed to convert URI to HTTPS");
                Err(StatusCode::BAD_REQUEST)
            }
        }
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], ports.http));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    let server = Server::bind(&addr).serve(redirect.into_make_service());

    if let Err(err) = server.await {
        eprintln!("Server error: {}", err);
    }
}
