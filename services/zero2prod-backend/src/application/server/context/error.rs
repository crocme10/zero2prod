use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize)]
pub enum Error {
    TBD {
        context: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::TBD { context } => {
                write!(fmt, "TBD: {context}")
            }
        }
    }
}

impl std::error::Error for Error {}
