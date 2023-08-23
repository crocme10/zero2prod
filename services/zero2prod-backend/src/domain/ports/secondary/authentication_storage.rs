use async_trait::async_trait;
use secrecy::Secret;
use uuid::Uuid;

use super::Error;
use crate::domain::Credentials;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AuthenticationStorage {
    async fn get_credentials(
        &self,
        username: &str,
    ) -> Result<Option<(Uuid, Secret<String>)>, Error>;

    async fn id_exists(&self, id: &Uuid) -> Result<bool, Error>;

    // Strre credentials (register new user)
    // TODO Maybe should return the id
    async fn store_credentials(
        &self,
        id: Uuid,
        email: &str,
        credentials: &Credentials,
    ) -> Result<(), Error>;

    async fn email_exists(&self, email: &str) -> Result<bool, Error>;

    async fn username_exists(&self, username: &str) -> Result<bool, Error>;
}
