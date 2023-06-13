use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::err_context::ErrorContextExt;
use crate::settings::DatabaseSettings;
use crate::storage::{Error, Storage};
use sqlx::postgres::{PgPool, PgPoolOptions};

#[derive(Debug, Clone)]
pub struct PostgresStorage {
    pub pool: PgPool,
    pub config: DatabaseSettings,
    pub conn_str: String,
    pub kind: PostgresStorageKind,
}

impl PostgresStorage {
    pub async fn new(config: DatabaseSettings, kind: PostgresStorageKind) -> Result<Self, Error> {
        let conn_str = config.connection_string();
        let pool = connect_with_conn_str(&conn_str, config.connection_timeout).await?;
        Ok(PostgresStorage {
            pool,
            config,
            conn_str,
            kind,
        })
    }
}

#[derive(Debug, Clone)]
pub enum PostgresStorageKind {
    Normal,
    Testing,
}

pub async fn connect_with_conn_str(conn_str: &str, timeout: u64) -> Result<PgPool, Error> {
    let pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_millis(timeout))
        .connect(conn_str)
        .await
        .context(format!(
            "Could not establish connection to {conn_str} with timeout {timeout}"
        ))?;

    Ok(pool)
}

#[async_trait]
impl Storage for PostgresStorage {
    async fn create_subscription(&self, username: String, email: String) -> Result<(), Error> {
        let pool = &self.pool;
        let _ = sqlx::query!(
        r#"INSERT INTO subscriptions (id, email, username, subscribed_at) VALUES ($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        email,
        username,
        Utc::now()
        )
        .execute(&*pool)
        .await
        .context(format!(
                "Could not create new subscription for {username}"
                ))?;

        Ok(())
    }
}