use async_trait::async_trait;
use common::err_context::ErrorContextExt;
use secrecy::{ExposeSecret, Secret};
use uuid::Uuid;

use super::PostgresStorage;
use crate::authentication::password::compute_password_hash;
use crate::domain::{
    ports::secondary::AuthenticationError, ports::secondary::AuthenticationStorage, Credentials,
};
use crate::utils::tracing::spawn_blocking_with_tracing;

#[async_trait]
impl AuthenticationStorage for PostgresStorage {
    #[tracing::instrument(name = "Getting credentials from postgres")]
    async fn get_credentials(
        &self,
        username: &str,
    ) -> Result<Option<(Uuid, Secret<String>)>, AuthenticationError> {
        let row: Option<_> = sqlx::query!(
            r#"
            SELECT id, password_hash
            FROM users
            WHERE username = $1
            "#,
            username,
        )
        .fetch_optional(&self.pool)
        .await
        .context("Could not retrieve credentials")?
        .map(|row| (row.id, Secret::new(row.password_hash)));

        Ok(row)
    }

    // We skip email and credentials in the log for security.
    #[tracing::instrument(name = "Storing credentials in postgres", skip(email, credentials))]
    async fn store_credentials(
        &self,
        id: Uuid,
        email: &str,
        credentials: &Credentials,
    ) -> Result<(), AuthenticationError> {
        let Credentials { username, password } = credentials.clone();
        let password_hash = spawn_blocking_with_tracing(move || compute_password_hash(password))
            .await
            .map_err(|_| AuthenticationError::Miscellaneous {
                context: "Could not spawn task to compute hash password".to_string(),
            })?
            .map_err(|_| AuthenticationError::Password {
                context: "Could not compute hash password".to_string(),
            })?;

        sqlx::query!(
            r#"INSERT INTO users (id, username, email, password_hash) VALUES ($1, $2, $3, $4)"#,
            id,
            username,
            email,
            password_hash.expose_secret(),
        )
        .execute(&self.pool)
        .await
        .context("Could not create credentials")?;

        Ok(())
    }

    #[tracing::instrument(name = "Checking user id exists")]
    async fn id_exists(&self, id: &Uuid) -> Result<bool, AuthenticationError> {
        let exist = sqlx::query_scalar!(r#"SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)"#, id,)
            .fetch_one(&self.pool)
            .await
            .context("Could not check id exists")?
            .unwrap();

        Ok(exist)
    }

    #[tracing::instrument(name = "Checking email exists")]
    async fn email_exists(&self, email: &str) -> Result<bool, AuthenticationError> {
        let exist = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)"#,
            email,
        )
        .fetch_one(&self.pool)
        .await
        .context("Could not check email exists")?
        .unwrap();

        Ok(exist)
    }

    #[tracing::instrument(name = "Checking username exists")]
    async fn username_exists(&self, username: &str) -> Result<bool, AuthenticationError> {
        let exist = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)"#,
            username
        )
        .fetch_one(&self.pool)
        .await
        .context("Could not check username exists")?
        .unwrap();

        Ok(exist)
    }
}
