use common::err_context::ErrorContext;
use serde::Serialize;
use std::fmt;

use crate::authentication::jwt::Error as JwtError;

#[derive(Clone, Serialize, Debug)]
pub enum Error {
    ContextNotFound,
    TokenNotFound,
    InvalidCredentials { context: String, source: JwtError },
    InvalidUserId { context: String },
}

impl From<ErrorContext<JwtError>> for Error {
    fn from(err: ErrorContext<JwtError>) -> Self {
        Error::InvalidCredentials {
            context: err.0,
            source: err.1,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ContextNotFound => {
                write!(fmt, "Context not found")
            }
            Error::TokenNotFound => {
                write!(fmt, "Token not found")
            }
            Error::InvalidCredentials { context, source } => {
                write!(fmt, "Invalid Credentials: {context} {source}")
            }
            Error::InvalidUserId { context } => {
                write!(fmt, "Invalid User ID: {context}")
            }
        }
    }
}

impl std::error::Error for Error {}
