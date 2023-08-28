use base64::{DecodeError, Engine};
use hyper::header::{HeaderMap, ToStrError, AUTHORIZATION};
use secrecy::Secret;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use std::{fmt, string::FromUtf8Error};

use crate::domain::Credentials;
use common::err_context::{ErrorContext, ErrorContextExt};

/// Extract user's credentials from a header with the basic authentication scheme.
pub fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, Error> {
    // The header value, if present, must be a valid UTF8 string
    let header_value = headers
        .get(AUTHORIZATION)
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

#[serde_as]
#[derive(Debug, Serialize)]
pub enum Error {
    MissingHeader {
        context: String,
    },
    InvalidAuthenticationString {
        context: String,
        #[serde_as(as = "DisplayFromStr")]
        source: ToStrError,
    },
    InvalidAuthenticationScheme {
        context: String,
    },
    Base64 {
        context: String,
        #[serde_as(as = "DisplayFromStr")]
        source: DecodeError,
    },
    CredentialString {
        context: String,
        #[serde_as(as = "DisplayFromStr")]
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

impl From<ErrorContext<ToStrError>> for Error {
    fn from(err: ErrorContext<ToStrError>) -> Self {
        Error::InvalidAuthenticationString {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<DecodeError>> for Error {
    fn from(err: ErrorContext<DecodeError>) -> Self {
        Error::Base64 {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<FromUtf8Error>> for Error {
    fn from(err: ErrorContext<FromUtf8Error>) -> Self {
        Error::CredentialString {
            context: err.0,
            source: err.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use fake::locales::*;
    use fake::Fake;
    use secrecy::ExposeSecret;
    use speculoos::prelude::*;

    use crate::domain::{Credentials, CredentialsGenerator};

    use super::*;

    #[tokio::test]
    async fn basic_authentication_should_correctly_extract_valid_credentials() {
        let credentials: Credentials = CredentialsGenerator(EN).fake();
        let segment = format!("Basic {}", credentials.encode());

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, segment.parse().unwrap());
        let res = basic_authentication(&headers).expect("valid credentials");
        assert_that(&res.username).is_equal_to(&credentials.username);
        assert_that(&res.password.expose_secret())
            .is_equal_to(credentials.password.expose_secret());
    }
}
