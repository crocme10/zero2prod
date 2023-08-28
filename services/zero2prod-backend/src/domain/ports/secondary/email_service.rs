/// Interface to a service for sending email.
use crate::domain::SubscriberEmail;
use async_trait::async_trait;
use common::err_context::ErrorContext;
use serde_with::{serde_as, DisplayFromStr};
use serde::{Deserialize, Serialize};
use std::fmt;

#[cfg(test)]
use mockall::predicate::*;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait EmailService {
    async fn send_email(&self, email: Email) -> Result<(), Error>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    pub to: SubscriberEmail,
    // from will be filled by the EmailService implementation.
    pub subject: String,
    pub html_content: String,
    pub text_content: String,
}

/// This is the error used by the email service
/// TODO Prune, not all enums are used.
#[serde_as]
#[derive(Debug, Serialize)]
pub enum Error {
    /// Connection issue with the database
    Connection {
        context: String,
        #[serde_as(as = "DisplayFromStr")]
        source: reqwest::Error,
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

impl From<ErrorContext<reqwest::Error>> for Error {
    fn from(err: ErrorContext<reqwest::Error>) -> Self {
        Error::Connection {
            context: format!("FIXME: {}", err.0),
            source: err.1,
        }
    }
}
