use async_trait::async_trait;
use secrecy::Secret;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::fmt;
use uuid::Uuid;

use crate::domain::{ConfirmedSubscriber, Credentials, NewSubscription, Subscription};
use common::err_context::ErrorContext;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Storage {
    /// Store a new subscription, and a token, and return the subscription
    async fn create_subscription_and_store_token(
        &self,
        subscription: &NewSubscription,
        token: &str,
    ) -> Result<Subscription, Error>;

    async fn get_subscription_by_email(&self, email: &str) -> Result<Option<Subscription>, Error>;

    async fn get_subscriber_id_by_token(&self, token: &str) -> Result<Option<Uuid>, Error>;

    async fn get_token_by_subscriber_id(&self, id: &Uuid) -> Result<Option<String>, Error>;

    /// Modify the status of the subscriber identified by id to 'confirmed'
    async fn confirm_subscriber_by_id_and_delete_token(&self, id: &Uuid) -> Result<(), Error>;

    /// Delete a previously stored token identified by a subscriber_id
    async fn delete_confirmation_token(&self, id: &Uuid) -> Result<(), Error>;

    async fn get_confirmed_subscribers_email(&self) -> Result<Vec<ConfirmedSubscriber>, Error>;

    async fn get_credentials(
        &self,
        username: &str,
    ) -> Result<Option<(Uuid, Secret<String>)>, Error>;

    async fn id_exists(&self, id: &Uuid) -> Result<bool, Error>;

    // Strre credentials (register new user)
    async fn store_credentials(
        &self,
        id: Uuid,
        email: &str,
        credentials: &Credentials,
    ) -> Result<(), Error>;

    async fn email_exists(&self, email: &str) -> Result<bool, Error>;

    async fn username_exists(&self, username: &str) -> Result<bool, Error>;
}

#[derive(Debug)]
pub enum Error {
    /// Error returned by sqlx
    Database {
        context: String,
        source: sqlx::Error,
    },
    /// Data store cannot be validated
    Validation {
        context: String,
    },
    /// Connection issue with the databas
    Connection {
        context: String,
        source: sqlx::Error,
    },
    Configuration {
        context: String,
    },
    Missing {
        context: String,
    },
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
            Error::Configuration { context } => {
                write!(fmt, "Database Configuration: {context}")
            }
            Error::Missing { context } => {
                write!(fmt, "Missing: {context}")
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

/// FIXME This is an oversimplified serialization for the Error.
/// I had to do this because some fields (source) where not 'Serialize'
impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Error", 1)?;
        match self {
            Error::Database { context, source: _ } => {
                state.serialize_field("description", context)?;
            }
            Error::Validation { context } => {
                state.serialize_field("description", context)?;
            }
            Error::Connection { context, source: _ } => {
                state.serialize_field("description", context)?;
            }
            Error::Configuration { context } => {
                state.serialize_field("description", context)?;
            }
            Error::Missing { context } => {
                state.serialize_field("description", context)?;
            }
        }
        state.end()
    }
}
