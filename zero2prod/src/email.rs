/// Interface to a service for sending email.
use crate::domain::SubscriberEmail;
use async_trait::async_trait;
use std::fmt;
use zero2prod_common::err_context::ErrorContext;

#[cfg(test)]
use mockall::{automock, mock, predicate::*};

// use zero2prod_common::err_context::ErrorContext;

#[derive(Debug)]
pub enum Error {
    /// Cannot connect to Email Service
    Connection {
        context: String,
        source: reqwest::Error,
    },
    /// Configuration Error for Email Service Client
    Configuration {
        context: String,
    },
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

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Email {
    async fn send_email(
        &self,
        recipient: &SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), Error>;
}
