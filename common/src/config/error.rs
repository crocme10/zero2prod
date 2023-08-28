use config::ConfigError;
use std::fmt;

use crate::err_context::ErrorContext;

#[derive(Debug)]
pub enum Error {
    Configuration {
        context: String,
        source: ConfigError,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Configuration { context, source } => {
                write!(
                    fmt,
                    "Could not create configuration: {context} | source: {source}"
                )
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<ErrorContext<ConfigError>> for Error {
    fn from(ctx: ErrorContext<ConfigError>) -> Error {
        Error::Configuration {
            context: ctx.0,
            source: ctx.1,
        }
    }
}
