use async_trait::async_trait;
use chrono::Utc;
use common::err_context::ErrorContextExt;
use common::settings::DatabaseSettings;
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::domain::{
    NewSubscription, SubscriberEmail, SubscriberName, Subscription, SubscriptionStatus,
};
use crate::storage::{Error, Storage};

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
    async fn create_subscription_and_store_token(
        &self,
        new_subscription: &NewSubscription,
        token: &str,
    ) -> Result<Subscription, Error> {
        // FIXME The following two statements should be part of a transaction.
        // But I'm not sure how to bring in a Transaction from my Exec?
        // Side stepping this issue for now, as maybe sqlx will bring a solution
        // soon.
        let mut conn = self.exec.lock().await;
        let id = Uuid::new_v4();
        // FIXME Use a RETURNING clause instead of using a subsequent SELECT
        sqlx::query!(
        r#"INSERT INTO subscriptions (id, email, username, subscribed_at, status) VALUES ($1, $2, $3, $4, $5)"#,
        id,
        new_subscription.email.as_ref(),
        new_subscription.username.as_ref(),
        Utc::now(),
        SubscriptionStatus::PendingConfirmation as SubscriptionStatus,
        )
        .execute(&mut **conn)
        .await
        .context(format!(
                "Could not create new subscription for {}", new_subscription.username.as_ref()
                ))?;

        sqlx::query!(
            r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id) VALUES ($1, $2)"#,
            token, id
        )
        .execute(&mut **conn)
        .await
        .context(format!("Could not store subscription token for subscriber id {id}"))?;
        let saved = sqlx::query!(
            r#"SELECT id, email, username, status::text FROM subscriptions WHERE id = $1"#,
            id
        )
        .fetch_one(&mut **conn)
        .await
        .context(format!("Could not get subscription for {id}"))?;
        let username = SubscriberName::parse(saved.username).map_err(|err| Error::Validation {
            context: format!("Invalid username: {err}"),
        })?;
        let email = SubscriberEmail::parse(saved.email).map_err(|err| Error::Validation {
            context: format!("Invalid email: {err}"),
        })?;
        let status =
            SubscriptionStatus::from_str(&saved.status.unwrap_or_default()).map_err(|err| {
                Error::Validation {
                    context: format!("Invalid status: {err}"),
                }
            })?;
        Ok(Subscription {
            id: saved.id,
            username,
            email,
            status,
        })
    }

    #[tracing::instrument(name = "Fetching a subscription by email in postgres")]
    async fn get_subscription_by_email(&self, email: &str) -> Result<Option<Subscription>, Error> {
        let mut conn = self.exec.lock().await;
        let saved = sqlx::query!(
            r#"SELECT id, email, username, status::text FROM subscriptions WHERE email = $1"#,
            email
        )
        .fetch_optional(&mut **conn)
        .await
        .context(format!("Could not get subscription for {email}"))?;
        tracing::info!("saved: {saved:?}");
        match saved {
            None => Ok(None),
            Some(rec) => {
                let username =
                    SubscriberName::parse(rec.username).map_err(|err| Error::Validation {
                        context: format!("Invalid username: {err}"),
                    })?;
                let email = SubscriberEmail::parse(rec.email).map_err(|err| Error::Validation {
                    context: format!("Invalid email: {err}"),
                })?;
                let status = SubscriptionStatus::from_str(&rec.status.unwrap_or_default())
                    .map_err(|err| Error::Validation {
                        context: format!("Invalid status: {err}"),
                    })?;
                Ok(Some(Subscription {
                    id: rec.id,
                    username,
                    email,
                    status,
                }))
            }
        }
    }

    #[tracing::instrument(name = "Fetching a subscriber id by token in postgres")]
    async fn get_subscriber_id_by_token(&self, token: &str) -> Result<Option<Uuid>, Error> {
        let mut conn = self.exec.lock().await;
        let saved = sqlx::query!(
            r#"SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1"#,
            token
        )
        .fetch_optional(&mut **conn)
        .await
        .context(format!("Could not get subscriber id for {token}"))?;
        tracing::info!("saved: {saved:?}");
        Ok(saved.map(|r| r.subscriber_id))
    }

    #[tracing::instrument(name = "Fetching a token using the subscriber's id in postgres")]
    async fn get_token_by_subscriber_id(&self, id: &Uuid) -> Result<Option<String>, Error> {
        // FIXME Move to transaction
        let mut conn = self.exec.lock().await;
        let saved = sqlx::query!(
            r#"SELECT subscription_token FROM subscription_tokens WHERE subscriber_id = $1"#,
            id
        )
        .fetch_optional(&mut **conn)
        .await
        .context(format!("Could not get token from subscriber id {id}"))?;
        Ok(saved.map(|r| r.subscription_token))
    }

    #[tracing::instrument(name = "Deleting subscription token")]
    async fn delete_confirmation_token(&self, id: &Uuid) -> Result<(), Error> {
        let mut conn = self.exec.lock().await;
        sqlx::query!(
            r#"DELETE FROM subscription_tokens WHERE subscriber_id = $1"#,
            id
        )
        .execute(&mut **conn)
        .await
        .context(format!(
            "Could not delete subscription token for subscriber id {id}"
        ))?;
        Ok(())
    }

    #[tracing::instrument(name = "Confirming subscriber")]
    async fn confirm_subscriber_by_id_and_delete_token(&self, id: &Uuid) -> Result<(), Error> {
        let mut conn = self.exec.lock().await;
        sqlx::query!(
            r#"UPDATE subscriptions SET status = $1 WHERE id = $2"#,
            SubscriptionStatus::Confirmed as SubscriptionStatus,
            id
        )
        .execute(&mut **conn)
        .await
        .context(format!("Could not confirm subscriber by id {id}"))?;
        sqlx::query!(
            r#"DELETE FROM subscription_tokens WHERE subscriber_id = $1"#,
            id
        )
        .execute(&mut **conn)
        .await
        .context(format!(
            "Could not delete subscription token for subscriber id {id}"
        ))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use common::settings::Settings;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::name::en::Name;
    use fake::Fake;
    use speculoos::prelude::*;
    use std::path::PathBuf;
    use std::sync::Arc;

    use crate::{
        domain::NewSubscription,
        opts::{Command, Opts},
        routes::subscriptions::SubscriptionRequest,
    };

    use super::*;

    #[tokio::test]
    async fn storage_should_store_and_retrieve_subscription() {
        // In this test we just store a subscription, and then try
        // to retrieve it using the email.
        let opts = Opts {
            config_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
                .join("config"),
            run_mode: Some("testing".to_string()),
            settings: vec![],
            cmd: Command::Run,
        };

        // And then build the configuration that would come from the command line arguments.
        let settings: Settings = opts.try_into().expect("settings");

        let storage = Arc::new(
            PostgresStorage::new(settings.database)
                .await
                .expect("Establishing a database connection"),
        );

        let username = Name().fake::<String>();
        let email = SafeEmail().fake::<String>();
        let request = SubscriptionRequest { username, email };
        let new_subscription = NewSubscription::try_from(request.clone()).unwrap();

        let email = new_subscription.email.clone();

        let token = 32.fake::<String>();
        let lhs = storage
            .create_subscription_and_store_token(&new_subscription, &token)
            .await
            .expect("storing subscription");

        let rhs = storage
            .get_subscription_by_email(email.as_ref())
            .await
            .expect("getting subscription");

        assert_that(&lhs).is_equal_to(&rhs.unwrap());
    }

    #[tokio::test]
    async fn storage_should_store_and_retrieve_subscriber_by_token() {
        // In this test we store a subscription, and then try to
        // retrieve the subscriber id given the token.
        let opts = Opts {
            config_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
                .join("config"),
            run_mode: Some("testing".to_string()),
            settings: vec![],
            cmd: Command::Run,
        };

        // And then build the configuration that would come from the command line arguments.
        let settings: Settings = opts.try_into().expect("settings");

        let storage = Arc::new(
            PostgresStorage::new(settings.database)
                .await
                .expect("Establishing a database connection"),
        );

        let username = Name().fake::<String>();
        let email = SafeEmail().fake::<String>();
        let request = SubscriptionRequest { username, email };
        let new_subscription = NewSubscription::try_from(request).unwrap();

        let token = 32.fake::<String>();
        let subscription = storage
            .create_subscription_and_store_token(&new_subscription, &token)
            .await
            .expect("storing subscription");

        let id = storage
            .get_subscriber_id_by_token(&token)
            .await
            .expect("getting subscriber id");

        assert_that(&id.unwrap()).is_equal_to(&subscription.id);
    }

    #[tokio::test]
    async fn storage_should_not_retrieve_subscriber_by_token_once_deleted() {
        // In this test we store a subscription,
        // Then we confirm the subscriber
        // We check that the subscriber's status is 'confirmed'
        // Finally we try to retrieve the subscriber id by the token,
        // which should be deleted from the subscription_token table.
        let opts = Opts {
            config_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
                .join("config"),
            run_mode: Some("testing".to_string()),
            settings: vec![],
            cmd: Command::Run,
        };

        // And then build the configuration that would come from the command line arguments.
        let settings: Settings = opts.try_into().expect("settings");

        let storage = Arc::new(
            PostgresStorage::new(settings.database)
                .await
                .expect("Establishing a database connection"),
        );

        let username = Name().fake::<String>();
        let email = SafeEmail().fake::<String>();
        let request = SubscriptionRequest { username, email };
        let new_subscription = NewSubscription::try_from(request).unwrap();

        let email = new_subscription.email.clone();
        let token = 32.fake::<String>();
        let subscription = storage
            .create_subscription_and_store_token(&new_subscription, &token)
            .await
            .expect("storing subscription");

        storage
            .confirm_subscriber_by_id_and_delete_token(&subscription.id)
            .await
            .expect("confirming subscriber id");

        let subscription = storage
            .get_subscription_by_email(email.as_ref())
            .await
            .expect("confirming subscriber id");

        assert_that(&subscription).is_some();

        let subscription = subscription.unwrap();

        assert_that(&subscription.status).is_equal_to(SubscriptionStatus::Confirmed);

        let id = storage
            .get_subscriber_id_by_token(&token)
            .await
            .expect("getting subscriber id");

        assert_that(&id).is_none();
    }
}