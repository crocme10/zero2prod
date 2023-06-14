use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;
use std::ops::{DerefMut, Deref};

use crate::err_context::ErrorContextExt;
use crate::settings::DatabaseSettings;
use crate::storage::{Error, Storage, Subscription};
use sqlx::postgres::{PgPool, PgPoolOptions};

pub enum Exec<'c> {
    Connection(sqlx::pool::PoolConnection<sqlx::Postgres>),
    Transaction(sqlx::Transaction<'c, sqlx::Postgres>)
}

impl<'c> Deref for Exec<'c> {
    type Target = sqlx::PgConnection;

    fn deref(&self) -> &Self::Target {
        match self {
            Exec::Connection(conn) => conn.deref(),
            Exec::Transaction(tx) => tx.deref(),
        }
    }
}

impl<'c> DerefMut for Exec<'c>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Exec::Connection(conn) => conn.deref_mut(),
            Exec::Transaction(tx) => tx.deref_mut(),
        }
    }
}

#[derive(Clone)]
pub struct PostgresStorage<'a> {
    pub pool: PgPool,
    pub exec: Arc<Exec<'a>>,
    pub config: DatabaseSettings,
    pub conn_str: String,
    pub kind: PostgresStorageKind,
}

impl PostgresStorage<'_>
{
    pub async fn new(config: DatabaseSettings, kind: PostgresStorageKind) -> Result<PostgresStorage<'static>, Error> {
        let conn_str = config.connection_string();
        let pool = connect_with_conn_str(&conn_str, config.connection_timeout).await?;
        let exec = match kind {
            PostgresStorageKind::Normal => Exec::Connection(pool.acquire().await.expect("acquire connection")),
            PostgresStorageKind::Testing => Exec::Transaction(pool.begin().await.expect("acquire transaction")),
        };
        Ok(PostgresStorage {
            pool,
            exec: Arc::new(exec),
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
impl Storage for PostgresStorage<'_>
{
    async fn create_subscription(&self, username: String, email: String) -> Result<(), Error> {
        let _ = sqlx::query!(
        r#"INSERT INTO subscriptions (id, email, username, subscribed_at) VALUES ($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        email,
        username,
        Utc::now()
        )
        .execute(&mut **self.exec)
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
