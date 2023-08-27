use std::fmt;

use common::err_context::ErrorContext;

#[derive(Debug)]
pub enum Error {
    Server {
        context: String,
        source: hyper::Error,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Server { context, source } => {
                write!(fmt, "Server: {context} | {source}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<ErrorContext<String, hyper::Error>> for Error {
    fn from(err: ErrorContext<String, hyper::Error>) -> Self {
        Error::Server {
            context: err.0,
            source: err.1,
        }
    }
}
