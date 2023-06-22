use async_trait::async_trait;
use chrono::Utc;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::domain::NewSubscription;
use crate::storage::{Error, Storage, Subscription};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use zero2prod_common::err_context::ErrorContextExt;
use zero2prod_common::settings::DatabaseSettings;

/// This is the executor type, which can be either a pool connection, or a transaction.
/// This is the sort of generic solution that I have found which allows me to use
/// either kind of connection, depending on the context:
/// a transaction in a testing environment,
/// a connection otherwise
#[derive(Debug)]
pub enum Exec<'c> {
    Connection(sqlx::pool::PoolConnection<sqlx::Postgres>),
    Transaction(sqlx::Transaction<'c, sqlx::Postgres>),
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

impl<'c> DerefMut for Exec<'c> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Exec::Connection(conn) => conn.deref_mut(),
            Exec::Transaction(tx) => tx.deref_mut(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PostgresStorage {
    pub pool: PgPool,
    pub exec: Arc<Mutex<Exec<'static>>>,
    pub config: DatabaseSettings,
    pub conn_options: PgConnectOptions,
}

impl PostgresStorage {
    pub async fn new(config: DatabaseSettings) -> Result<PostgresStorage, Error> {
        let pool = connect_with_options(&config).await?;
        tracing::debug!("Connected Postgres Pool to {}", config.connection_string());
        let exec = match config.executor.as_str() {
            "connection" => {
                tracing::info!("PostgresStorage: Creating a connection");
                Exec::Connection(pool.acquire().await.expect("acquire connection"))
            }
            "transaction" => {
                tracing::info!("PostgresStorage: Creating a transaction");
                Exec::Transaction(pool.begin().await.expect("acquire transaction"))
            }
            _ => {
                tracing::warn!("PostgresStorage: Unrecognized executor kind");
                return Err(Error::Configuration {
                    context: "Unrecognized error kind".to_string(),
                });
            }
        };
        let conn_options = config.connect_options();
        Ok(PostgresStorage {
            pool,
            exec: Arc::new(Mutex::new(exec)),
            config,
            conn_options,
        })
    }
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

pub async fn connect_with_options(config: &DatabaseSettings) -> Result<PgPool, Error> {
    let options = config.connect_options();
    let pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_millis(config.connection_timeout))
        .connect_with(options)
        .await
        .context(format!(
            "Could not establish connection to {} with timeout {}",
            config.connection_string(),
            config.connection_timeout
        ))?;

    Ok(pool)
}

#[async_trait]
impl Storage for PostgresStorage {
    #[tracing::instrument(name = "Storing a new subscription in postgres")]
    async fn create_subscription(&self, subscription: &NewSubscription) -> Result<(), Error> {
        let mut conn = self.exec.lock().await;
        let _ = sqlx::query!(
        r#"INSERT INTO subscriptions (id, email, username, subscribed_at) VALUES ($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        &subscription.email,
        subscription.username.as_ref(),
        Utc::now()
        )
        .execute(&mut **conn)
        .await
        .context(format!(
                "Could not create new subscription for {}", subscription.username.as_ref()
                ))?;

        Ok(())
    }

    #[tracing::instrument(name = "Fetching a subscription by username in postgres")]
    async fn get_subscription_by_username(
        &self,
        username: &str,
    ) -> Result<Option<Subscription>, Error> {
        tracing::info!("Fetching subscription by username {username}");
        let mut conn = self.exec.lock().await;
        let saved = sqlx::query!(
            r#"SELECT email, username FROM subscriptions WHERE username = $1"#,
            username
        )
        .fetch_optional(&mut **conn)
        .await
        .context(format!("Could not get subscription for {username}"))?;
        tracing::info!("saved: {saved:?}");
        Ok(saved.map(|rec| Subscription {
            username: rec.username,
            email: rec.email,
        }))
    }
}
