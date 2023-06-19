use serde::{Deserialize, Serialize};
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
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub network: NetworkSettings,
    pub database: DatabaseSettings,
    pub mode: String,
}
