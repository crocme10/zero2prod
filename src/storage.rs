use async_trait::async_trait;
use std::fmt;

use crate::err_context::ErrorContext;

#[derive(Debug)]
pub enum Error {
    DBConnection {
        context: String,
        source: sqlx::Error,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::DBConnection { context, source } => {
                write!(fmt, "Database: {context} | {source}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<ErrorContext<String, sqlx::Error>> for Error {
    fn from(err: ErrorContext<String, sqlx::Error>) -> Self {
        match err.1 {
            sqlx::Error::PoolTimedOut => Error::DBConnection {
                context: "PostgreSQL Storage: Connection Timeout".to_string(),
                source: err.1,
            },
            _ => Error::DBConnection {
                context: "PostgreSQL Storage: Could not establish a connection".to_string(),
                source: err.1,
            },
        }
    }
}

#[async_trait]
pub trait Storage {
    async fn create_subscription(&self, username: String, email: String) -> Result<(), Error>;
}
