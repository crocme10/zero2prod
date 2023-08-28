use common::err_context::ErrorContext;
use std::fmt;

use super::listener::Error as ListenerError;
use crate::domain::ports::secondary::{AuthenticationError, EmailError, SubscriptionError};
use crate::services::postgres::Error as PostgresError;

#[derive(Debug)]
pub enum Error {
    Listener {
        context: String,
        source: ListenerError,
    },
    Postgres {
        context: String,
        source: PostgresError,
    },
    Authentication {
        context: String,
        source: AuthenticationError,
    },
    Subscription {
        context: String,
        source: SubscriptionError,
    },
    Email {
        context: String,
        source: EmailError,
    },
    Server {
        context: String,
        source: hyper::Error,
    },
    Path {
        context: String,
        source: std::io::Error,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Listener { context, source } => {
                write!(fmt, "Could not build TCP listener: {context} | {source}")
            }
            Error::Postgres { context, source } => {
                write!(fmt, "Storage Error: {context} | {source}")
            }
            Error::Authentication { context, source } => {
                write!(fmt, "Authentication Error: {context} | {source}")
            }
            Error::Subscription { context, source } => {
                write!(fmt, "Subscription Error: {context} | {source}")
            }
            Error::Email { context, source } => {
                write!(fmt, "Email Error: {context} | {source}")
            }
            Error::Server { context, source } => {
                write!(fmt, "Application Server Error: {context} | {source}")
            }
            Error::Path { context, source } => {
                write!(fmt, "IO Error: {context} | {source}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<ErrorContext<AuthenticationError>> for Error {
    fn from(err: ErrorContext<AuthenticationError>) -> Self {
        Error::Authentication {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<PostgresError>> for Error {
    fn from(err: ErrorContext<PostgresError>) -> Self {
        Error::Postgres {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<SubscriptionError>> for Error {
    fn from(err: ErrorContext<SubscriptionError>) -> Self {
        Error::Subscription {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<EmailError>> for Error {
    fn from(err: ErrorContext<EmailError>) -> Self {
        Error::Email {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<ListenerError>> for Error {
    fn from(err: ErrorContext<ListenerError>) -> Self {
        Error::Listener {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<hyper::Error>> for Error {
    fn from(err: ErrorContext<hyper::Error>) -> Self {
        Error::Server {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<std::io::Error>> for Error {
    fn from(err: ErrorContext<std::io::Error>) -> Self {
        Error::Path {
            context: err.0,
            source: err.1,
        }
    }
}
