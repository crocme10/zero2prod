use async_trait::async_trait;
use std::fmt;
use uuid::Uuid;

use crate::domain::NewSubscription;
use zero2prod_common::err_context::ErrorContext;

#[derive(Debug)]
pub enum Error {
    Database {
        context: String,
        source: sqlx::Error,
    },
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
            Error::Database { context, source } => {
                write!(fmt, "Database: {context} | {source}")
            }
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
                context: format!("PostgreSQL Storage: Connection Timeout: {}", err.0),
                source: err.1,
            },
            sqlx::Error::Database(_) => Error::Database {
                context: format!("PostgreSQL Storage: Database: {}", err.0),
                source: err.1,
            },
            _ => Error::Connection {
                context: format!(
                    "PostgreSQL Storage: Could not establish a connection: {}",
                    err.0
                ),
                source: err.1,
            },
        }
    }
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Storage {
    async fn create_subscription(&self, subscription: &NewSubscription) -> Result<(), Error>;
    async fn get_subscription_by_username(
        &self,
        username: &str,
    ) -> Result<Option<Subscription>, Error>;

    async fn get_subscriber_id_by_token(&self, token: &str) -> Result<Option<Uuid>, Error>;

    async fn confirm_subscriber_by_id(&self, id: &Uuid) -> Result<(), Error>;
}

#[derive(Debug)]
pub struct Subscription {
    pub username: String,
    pub email: String,
    pub status: String,
}
