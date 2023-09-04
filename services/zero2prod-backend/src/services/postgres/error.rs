use common::err_context::ErrorContext;
use common::settings::Error as SettingsError;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize)]
pub enum Error {
    /// Error returned by sqlx
    Database {
        context: String,
        source: String,
    },
    Validation {
        context: String,
    },
    /// Connection issue with the database
    Connection {
        context: String,
        source: String,
    },
    Configuration {
        context: String,
        source: SettingsError,
    },
    IO {
        context: String,
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Database { context, source } => {
                write!(fmt, "Database: {context} | {source}")
            }
            Error::Validation { context } => {
                write!(fmt, "Data: {context}")
            }
            Error::Connection { context, source } => {
                write!(fmt, "Database Connection: {context} | {source}")
            }
            Error::Configuration { context, source } => {
                write!(fmt, "Database Configuration: {context} | {source}")
            }
            Error::IO { context } => {
                write!(fmt, "IO Error: {context}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<ErrorContext<sqlx::Error>> for Error {
    fn from(err: ErrorContext<sqlx::Error>) -> Self {
        match err.1 {
            sqlx::Error::PoolTimedOut => Error::Connection {
                context: format!("PostgreSQL Storage: Connection Timeout: {}", err.0),
                source: err.1.to_string(),
            },
            sqlx::Error::Database(_) => Error::Database {
                context: format!("PostgreSQL Storage: Database: {}", err.0),
                source: err.1.to_string(),
            },
            _ => Error::Connection {
                context: format!(
                    "PostgreSQL Storage: Could not establish a connection: {}",
                    err.0
                ),
                source: err.1.to_string(),
            },
        }
    }
}

impl From<ErrorContext<SettingsError>> for Error {
    fn from(err: ErrorContext<SettingsError>) -> Self {
        Error::Configuration {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<std::io::Error>> for Error {
    fn from(err: ErrorContext<std::io::Error>) -> Self {
        Error::IO {
            context: format!("{}: {}", err.0, err.1)
        }
    }
}
