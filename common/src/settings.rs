use serde::{Deserialize, Serialize};
use serde_aux::field_attributes::deserialize_number_from_string;
use serde_with::{serde_as, DisplayFromStr};
use sqlx::postgres::PgConnectOptions;
use sqlx::postgres::PgSslMode;
use std::fmt;
use std::path::PathBuf;

use crate::config::{merge_configuration, Error as ConfigError};
use crate::err_context::{ErrorContext, ErrorContextExt};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationSettings {
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub http: u16,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
    pub connection_timeout: u64,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }

    pub fn connect_options(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            // Try an encrypted connection, fallback
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password)
            .database(&self.database_name)
            .port(self.port)
            .ssl_mode(ssl_mode)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailClientSettings {
    /// URL of the Email Service the client connects to.
    pub server_url: String,
    pub sender_email: String,
    pub authorization_token: String,
    pub timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingSettings {
    pub level: String,
    pub jaeger: Option<JaegerSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JaegerSettings {
    pub service_name: String,
    pub endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
    pub email_client: EmailClientSettings,
    pub tracing: TracingSettings,
    pub mode: String,
}

async fn database_settings_from_mode(mode: &str) -> Result<DatabaseSettings, Error> {
    let config_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("config");

    let config = merge_configuration(
        config_dir.as_ref(),
        &["database"],
        mode,
        "ZERO2PROD",
        vec![],
    )
    .context(format!("Could not get database {mode} settings"))?;

    let settings = config
        .get("database")
        .context(format!("Invalid database {mode} settings"))?;

    Ok(settings)
}

async fn tracing_settings_from_mode(mode: &str) -> Result<TracingSettings, Error> {
    let config_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("config");

    let config = merge_configuration(config_dir.as_ref(), &["tracing"], mode, "ZERO2PROD", vec![])
        .context(format!(
            "Could not get tracing '{mode}' settings in {}",
            config_dir.display()
        ))?;

    let settings = config.get("tracing").context(format!(
        "Invalid tracing '{mode}' settings in {}",
        config_dir.display()
    ))?;

    Ok(settings)
}

pub async fn database_root_settings() -> Result<DatabaseSettings, Error> {
    database_settings_from_mode("root").await
}

pub async fn database_dev_settings() -> Result<DatabaseSettings, Error> {
    database_settings_from_mode("dev").await
}

pub async fn tracing_root_settings() -> Result<TracingSettings, Error> {
    tracing_settings_from_mode("root").await
}

pub async fn tracing_dev_settings() -> Result<TracingSettings, Error> {
    tracing_settings_from_mode("dev").await
}

#[serde_as]
#[derive(Debug, Serialize)]
pub enum Error {
    Configuration {
        context: String,
        #[serde_as(as = "DisplayFromStr")]
        source: ConfigError,
    },
    Settings {
        context: String,
        #[serde_as(as = "DisplayFromStr")]
        source: ::config::ConfigError,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Configuration { context, source } => {
                write!(fmt, "Configuration: {context} | {source}")
            }
            Error::Settings { context, source } => {
                write!(fmt, "Settings: {context} | {source}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<ErrorContext<::config::ConfigError>> for Error {
    fn from(err: ErrorContext<::config::ConfigError>) -> Self {
        Error::Settings {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<ConfigError>> for Error {
    fn from(err: ErrorContext<ConfigError>) -> Self {
        Error::Configuration {
            context: err.0,
            source: err.1,
        }
    }
}
