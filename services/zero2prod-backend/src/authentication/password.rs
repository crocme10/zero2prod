use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};
use secrecy::{ExposeSecret, Secret};
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use std::fmt;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::ports::secondary::{AuthenticationError, AuthenticationStorage};
use crate::domain::Credentials;
use crate::telemetry::spawn_blocking_with_tracing;
use common::err_context::{ErrorContext, ErrorContextExt};

// TODO This should really be a trait and an implementation...
// validate_credentials could be a free function, but for mocking
// it should be either a struct or a trait.
pub struct Authenticator {
    pub storage: Arc<dyn AuthenticationStorage + Send + Sync>,
}

#[cfg_attr(test, mockall::automock)]
impl Authenticator {
    #[tracing::instrument(
    name = "Validating Credentials"
    skip(self),
    fields(
        request_id = %Uuid::new_v4(),
    )
)]
    pub async fn validate_credentials(&self, credentials: &Credentials) -> Result<Uuid, Error> {
        let Credentials { username, password } = credentials.clone();

        let mut id = None;
        let mut expected_password_hash = Secret::new(
            "$argon2id$v=19$m=15000,t=2,p=1$\
        gZiV/M1gPc22ElAH/Jh1Hw$\
        CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
                .to_string(),
        );

        if let Some((stored_user_id, stored_password_hash)) = self
            .storage
            .get_credentials(&username)
            .await
            .context("Retrieving credentials for validation")?
        {
            id = Some(stored_user_id);
            expected_password_hash = stored_password_hash
        }

        spawn_blocking_with_tracing(move || {
            verify_password_hash(id, expected_password_hash, password)
        })
        .await
        .map_err(|_| Error::Miscellaneous {
            user: id,
            context: "Could not spawn blocking task".to_string(),
        })?
        .map_err(|_| Error::InvalidCredentials {
            user: id,
            context: "Could not verify password".to_string(),
        })?;

        id.ok_or_else(|| Error::InvalidCredentials {
            user: None,
            context: "Could not verify password".to_string(),
        })
    }
}

#[tracing::instrument(
    name = "Verify password hash"
    skip(expected_password_hash, password_candidate)
)]
fn verify_password_hash(
    user: Option<Uuid>,
    expected_password_hash: Secret<String>,
    password_candidate: Secret<String>,
) -> Result<(), Error> {
    let expected_password_hash = PasswordHash::new(expected_password_hash.expose_secret())
        .map_err(|_| Error::Miscellaneous {
            user,
            context: "Could not compute password hash".to_string(),
        })?;

    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .map_err(|_| {
            tracing::info!("argon2 could not verify password");
            Error::InvalidCredentials {
                user,
                context: "Password verification".to_string(),
            }
        })?;
    Ok(())
}

pub fn compute_password_hash(password: Secret<String>) -> Result<Secret<String>, Error> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let argon_params = Params::new(15000, 2, 1, None).context("Creating hashing parameters")?;

    let hasher = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params);
    let password_hash = hasher
        .hash_password(password.expose_secret().as_bytes(), &salt)
        .context("Hashing password")?;

    Ok(Secret::new(password_hash.to_string()))
}

#[serde_as]
#[derive(Clone, Debug, Serialize)]
pub enum Error {
    InvalidCredentials {
        user: Option<Uuid>,
        context: String,
    },
    Miscellaneous {
        user: Option<Uuid>,
        context: String,
    },
    Hasher {
        context: String,
        #[serde_as(as = "DisplayFromStr")]
        source: argon2::Error,
    },
    Hashing {
        context: String,
        #[serde_as(as = "DisplayFromStr")]
        source: argon2::password_hash::errors::Error,
    },
    // Error occurs when we try to retrieve credentials from Storage.
    Storage {
        context: String,
        source: AuthenticationError,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidCredentials { user, context } => {
                write!(
                    fmt,
                    "Invalid Credentials: {} {context} ",
                    user.map_or_else(|| "none".to_string(), |id| id.to_string())
                )
            }
            Error::Miscellaneous { user, context } => {
                write!(
                    fmt,
                    "Unexpected Error: {} {context} ",
                    user.map_or_else(|| "none".to_string(), |id| id.to_string())
                )
            }
            Error::Hasher { context, source } => {
                write!(fmt, "Hasher Error: {context} | {source}")
            }
            Error::Hashing { context, source } => {
                write!(fmt, "Hashing Error: {context} | {source}")
            }
            Error::Storage { context, source } => {
                write!(fmt, "Storage Error: {context} | {source}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<ErrorContext<AuthenticationError>> for Error {
    fn from(err: ErrorContext<AuthenticationError>) -> Self {
        Error::Storage {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<argon2::Error>> for Error {
    fn from(err: ErrorContext<argon2::Error>) -> Self {
        Error::Hasher {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<argon2::password_hash::errors::Error>> for Error {
    fn from(err: ErrorContext<argon2::password_hash::errors::Error>) -> Self {
        Error::Hashing {
            context: err.0,
            source: err.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use fake::locales::*;
    use fake::Fake;
    use speculoos::prelude::*;
    use uuid::Uuid;

    use crate::domain::ports::secondary::authentication_storage::MockAuthenticationStorage;
    use crate::domain::{Credentials, CredentialsGenerator};

    use super::*;

    #[tokio::test]
    async fn authenticator_should_call_get_credentials() {
        let credentials: Credentials = CredentialsGenerator(EN).fake();
        let hashed_password = compute_password_hash(credentials.password.clone()).unwrap();
        let id = Uuid::new_v4();
        let mut storage_mock = MockAuthenticationStorage::new();
        storage_mock
            .expect_get_credentials()
            .return_once(move |_| Ok(Some((id, hashed_password))));

        let authenticator = Authenticator {
            storage: Arc::new(storage_mock),
        };

        let res = authenticator
            .validate_credentials(&credentials)
            .await
            .expect("valid credentials");

        assert_that(&res).is_equal_to(id);
    }
}
