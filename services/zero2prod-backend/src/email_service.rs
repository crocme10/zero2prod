/// Interface to a service for sending email.
use crate::domain::SubscriberEmail;
use async_trait::async_trait;
use common::err_context::ErrorContext;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};

use std::fmt;

#[cfg(test)]
use mockall::predicate::*;

// use zero2prod_common::err_context::ErrorContext;

#[derive(Debug)]
pub enum Error {
    /// Cannot connect to Email Service
    Connection {
        context: String,
        source: reqwest::Error,
    },
    /// Configuration Error for Email Service Client
    Configuration { context: String },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Connection { context, source } => {
                write!(fmt, "Email Service Connection: {context} | {source}")
            }
            Error::Configuration { context } => {
                write!(fmt, "Email Service Configuration: {context}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<ErrorContext<String, reqwest::Error>> for Error {
    fn from(err: ErrorContext<String, reqwest::Error>) -> Self {
        Error::Connection {
            context: err.0,
            source: err.1,
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
            Error::Configuration { context } => {
                state.serialize_field("description", context)?;
            }
            Error::Connection { context, source: _ } => {
                state.serialize_field("description", context)?;
            }
        }
        state.end()
    }
}

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
