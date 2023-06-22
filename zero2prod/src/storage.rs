use async_trait::async_trait;
use std::fmt;

use zero2prod_common::err_context::ErrorContext;

#[derive(Debug)]
pub enum Error {
    Connection {
        context: String,
        source: sqlx::Error,
    },
    Configuration {
        context: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Connection { context, source } => {
                write!(fmt, "Database Connection: {context} | {source}")
            }
            Error::Configuration { context } => {
                write!(fmt, "Database Configuration: {context}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<ErrorContext<String, sqlx::Error>> for Error {
    fn from(err: ErrorContext<String, sqlx::Error>) -> Self {
        match err.1 {
            sqlx::Error::PoolTimedOut => Error::Connection {
                context: format!("PostgreSQL Storage: Connection Timeout: {err.0}"),
                source: err.1,
            },
            _ => Error::Connection {
                context: format!("PostgreSQL Storage: Could not establish a connection: {err.0}"),
                source: err.1,
            },
        }
    }
}

#[async_trait]
pub trait Storage {
    async fn create_subscription(&self, username: String, email: String) -> Result<(), Error>;
    async fn get_subscription_by_username(
        &self,
        username: &str,
    ) -> Result<Option<Subscription>, Error>;
}

#[derive(Debug)]
pub struct Subscription {
    pub username: String,
    pub email: String,
}
