use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use std::fmt;

use common::err_context::ErrorContext;

#[serde_as]
#[derive(Debug, Serialize)]
pub enum Error {
    Server {
        context: String,
        #[serde_as(as = "DisplayFromStr")]
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

impl From<ErrorContext<hyper::Error>> for Error {
    fn from(err: ErrorContext<hyper::Error>) -> Self {
        Error::Server {
            context: err.0,
            source: err.1,
        }
    }
}
