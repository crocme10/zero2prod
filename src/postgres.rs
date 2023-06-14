use async_trait::async_trait;
use chrono::Utc;
use sqlx::Acquire;
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::err_context::ErrorContextExt;
use crate::settings::DatabaseSettings;
use crate::storage::{Error, Storage, Subscription};
use sqlx::postgres::{PgPool, PgPoolOptions};

#[derive(Debug, Clone)]
pub struct PostgresStorage<E> {
    pub pool: PgPool,
    pub exec: E,
    pub config: DatabaseSettings,
    pub conn_str: String,
    pub kind: PostgresStorageKind,
}

impl<E> PostgresStorage<E>
where
    for<'e> &'e mut E: PgExecutor<'e>,
{
    pub async fn new(config: DatabaseSettings, kind: PostgresStorageKind) -> Result<Self, Error> {
        let conn_str = config.connection_string();
        let pool = connect_with_conn_str(&conn_str, config.connection_timeout).await?;
        let exec = match kind {
            PostgresStorageKind::Normal => pool.acquire().await.expect("acquire connection"),
            PostgresStorageKind::Testing => pool.begin().await.expect("acquire transaction"),
        };
        Ok(PostgresStorage {
            pool,
            exec,
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
impl<E> Storage for PostgresStorage<E>
where
    for<'e> &'e mut E: PgExecutor<'e>,
{
    async fn create_subscription(&self, username: String, email: String) -> Result<(), Error> {
        let pool = &self.pool;
        let _ = sqlx::query!(
        r#"INSERT INTO subscriptions (id, email, username, subscribed_at) VALUES ($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        email,
        username,
        Utc::now()
        )
        .execute(&self.exec)
        .await
        .context(format!(
                "Could not create new subscription for {username}"
                ))?;

        Ok(())
    }

    async fn get_subscription_by_username(
        &self,
        username: &str,
    ) -> Result<Option<Subscription>, Error> {
        let pool = &self.pool;
        let saved = sqlx::query!(
            r#"SELECT email, username FROM subscriptions WHERE username = $1"#,
            username
        )
        .fetch_optional(&*pool)
        .await
        .context(format!("Could not get subscription for {username}"))?;
        Ok(saved.map(|rec| Subscription {
            username: rec.username,
            email: rec.email,
        }))
    }
}
