use async_trait::async_trait;
use common::err_context::ErrorContext;
use secrecy::Secret;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use std::fmt;
use uuid::Uuid;

use crate::domain::Credentials;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AuthenticationStorage {
    async fn get_credentials(
        &self,
        username: &str,
    ) -> Result<Option<(Uuid, Secret<String>)>, Error>;

    async fn id_exists(&self, id: &Uuid) -> Result<bool, Error>;

    // Strre credentials (register new user)
    // TODO Maybe should return the id
    async fn store_credentials(
        &self,
        id: Uuid,
        email: &str,
        credentials: &Credentials,
    ) -> Result<(), Error>;

    async fn email_exists(&self, email: &str) -> Result<bool, Error>;

    async fn username_exists(&self, username: &str) -> Result<bool, Error>;
}

#[serde_as]
#[derive(Debug, Serialize)]
pub enum Error {
    /// Error returned by sqlx
    // *
    Database {
        context: String,
        #[serde_as(as = "DisplayFromStr")]
        source: sqlx::Error,
    },
    /// Data store cannot be validated
    // Validation {
    //     context: String,
    // },
    /// Connection issue with the database
    // *
    Connection {
        context: String,
        #[serde_as(as = "DisplayFromStr")]
        source: sqlx::Error,
    },
    // Configuration {
    //     context: String,
    // },
    Miscellaneous {
        context: String,
    },
    Password {
        context: String,
        // We could put `source: PasswordError`, but That
        // would make the definition of the error circular:
        // it would depend on pass:error, which depends on 
        // error
    },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Database { context, source } => {
                write!(fmt, "Database: {context} | {source}")
            }
            // Error::Validation { context } => {
            //     write!(fmt, "Data: {context}")
            // }
            Error::Connection { context, source } => {
                write!(fmt, "Database Connection: {context} | {source}")
            }
            // Error::Configuration { context } => {
            //     write!(fmt, "Database Configuration: {context}")
            // }
            Error::Miscellaneous { context } => {
                write!(fmt, "Miscellaneous: {context}")
            }
            Error::Password { context } => {
                write!(fmt, "Password Error: {context}")
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
            _ => Error::Database {
                context: format!(
                    "PostgreSQL Storage: Miscellaneous: {}",
                    err.0
                ),
                source: err.1,
            },
        }
    }
}

