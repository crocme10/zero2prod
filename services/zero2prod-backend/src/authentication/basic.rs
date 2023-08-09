use base64::{DecodeError, Engine};
use hyper::header::{HeaderMap, ToStrError};
use secrecy::Secret;
use std::{fmt, string::FromUtf8Error};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

use crate::domain::Credentials;
use common::err_context::{ErrorContext, ErrorContextExt};

/// Extract user's credentials from a header with the basic authentication scheme.
pub fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, Error> {
    // The header value, if present, must be a valid UTF8 string
    let header_value = headers
        .get("Authorization")
        .ok_or_else(|| Error::MissingHeader {
            context: "The Authorization Header was missing".to_string(),
        })?
        .to_str()
        .context("The 'Authorization' header was not a valid UTF8 string.".to_string())?;

    let base64encoded_segment =
        header_value
            .strip_prefix("Basic ")
            .ok_or_else(|| Error::InvalidAuthenticationScheme {
                context: "The Authorization scheme was not 'Basic'".to_string(),
            })?;

    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(base64encoded_segment)
        .context("Failed to base64-decode 'Basic' credentials.".to_string())?;

    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("The decoded credential string is not valid UTF8.".to_string())?;

    // Split into two segments, using ':' as delimiter
    let mut credentials = decoded_credentials.splitn(2, ':');
    let username = credentials
        .next()
        .ok_or_else(|| Error::InvalidCredentials {
            context: "The username was missing in 'Basic' authorization".to_string(),
        })?
        .to_string();
    let password = credentials
        .next()
        .ok_or_else(|| Error::InvalidCredentials {
            context: "The password was missing in 'Basic' authorization".to_string(),
        })?
        .to_string();
    Ok(Credentials {
        username,
        password: Secret::new(password),
    })
}

#[derive(Debug)]
pub enum Error {
    MissingHeader {
        context: String,
    },
    InvalidAuthenticationString {
        context: String,
        source: ToStrError,
    },
    InvalidAuthenticationScheme {
        context: String,
    },
    Base64 {
        context: String,
        source: DecodeError,
    },
    CredentialString {
        context: String,
        source: FromUtf8Error,
    },
    InvalidCredentials {
        context: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MissingHeader { context } => {
                write!(fmt, "Missing Authentication: {context} ")
            }
            Error::InvalidAuthenticationString { context, source } => {
                write!(fmt, "Invalid Authentication Content: {context} | {source}")
            }
            Error::InvalidAuthenticationScheme { context } => {
                write!(fmt, "Invalid Authentication Scheme: {context} ")
            }
            Error::Base64 { context, source } => {
                write!(fmt, "Base64 Decode: {context} | {source}")
            }
            Error::CredentialString { context, source } => {
                write!(fmt, "Invalid UTF8 String: {context} | {source}")
            }
            Error::InvalidCredentials { context } => {
                write!(fmt, "Invalid Credentials: {context} ")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<ErrorContext<String, ToStrError>> for Error {
    fn from(err: ErrorContext<String, ToStrError>) -> Self {
        Error::InvalidAuthenticationString {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<String, DecodeError>> for Error {
    fn from(err: ErrorContext<String, DecodeError>) -> Self {
        Error::Base64 {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<String, FromUtf8Error>> for Error {
    fn from(err: ErrorContext<String, FromUtf8Error>) -> Self {
        Error::CredentialString {
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
            Error::MissingHeader { context } => {
                state.serialize_field("description", context)?;
            }
            Error::InvalidAuthenticationString { context, source: _ } => {
                state.serialize_field("description", context)?;
            }
            Error::InvalidAuthenticationScheme { context } => {
                state.serialize_field("description", context)?;
            }
            Error::Base64 { context, source: _ } => {
                state.serialize_field("description", context)?;
            }
            Error::CredentialString { context, source: _ } => {
                state.serialize_field("description", context)?;
            }
            Error::InvalidCredentials { context } => {
                state.serialize_field("description", context)?;
            }
        }
        state.end()
    }
}


