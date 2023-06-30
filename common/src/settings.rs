use serde::{Deserialize, Serialize};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::PgConnectOptions;
use sqlx::postgres::PgSslMode;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Building {
        context: String,
        source: crate::config::Error,
    },
    Deserializing {
        context: String,
        source: ::config::ConfigError,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Building { context, source } => {
                write!(fmt, "Could not build client request: {context} | {source}")
            }
            Error::Deserializing { context, source } => {
                write!(fmt, "Could not build client request: {context} | {source}")
            }
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSettings {
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
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
    pub executor: String,
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
pub struct Settings {
    pub network: NetworkSettings,
    pub database: DatabaseSettings,
    pub email_client: EmailClientSettings,
    pub mode: String,
}
