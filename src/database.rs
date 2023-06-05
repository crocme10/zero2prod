use sqlx::postgres::{PgPool, PgPoolOptions};
use std::fmt;

use crate::err_context::{ErrorContext, ErrorContextExt};

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
                context: format!("PostgreSQL Storage: Could not establish a connection",),
                source: err.1,
            },
        }
    }
}

pub async fn connect_with_conn_str(conn_str: &str, timeout: u64) -> Result<PgPool, Error> {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_millis(timeout))
        .connect(conn_str)
        .await
        .context(format!(
            "Could not establish connection to {conn_str} with timeout {timeout}"
        ))
        .map_err(|err| err.into())
}
