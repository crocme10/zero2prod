/// Implementation of authentication_store and subscriptions_store using postgres
mod authentication;
mod error;
mod subscription;

pub use self::error::Error;

use common::config;
use common::err_context::ErrorContextExt;
use common::settings::DatabaseSettings;
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PostgresStorage {
    pub pool: PgPool,
    pub config: DatabaseSettings,
    pub conn_options: PgConnectOptions,
}

impl PostgresStorage {
    pub async fn new(config: DatabaseSettings) -> Result<PostgresStorage, Error> {
        let pool = connect_with_options(&config).await?;
        tracing::debug!("Connected Postgres Pool to {}", config.connection_string());
        let conn_options = config.connect_options();
        Ok(PostgresStorage {
            pool,
            config,
            conn_options,
        })
    }

    pub async fn exec_file(&self, file: &str) -> Result<(), Error> {
        let content = fs::read_to_string(file).context("Unable to read file for execution")?;

        let sqls: Vec<&str> = content.split(';').collect();

        for sql in sqls {
            sqlx::query(sql)
                .execute(&self.pool)
                .await
                .context("Unable to execute")?;
        }

        Ok(())
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

pub async fn init_dev_db() -> Result<PostgresStorage, Error> {
    tracing::info!("Initializing dev db");
    let _ = init_root().await?;
    init_dev().await
}

async fn database_settings_from_mode(mode: &str) -> Result<DatabaseSettings, Error> {
    let config_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("config");

    config::merge_configuration(
        config_dir.as_ref(),
        &["database"],
        mode,
        "ZERO2PROD",
        vec![],
    )
    .map_err(|_| Error::Configuration {
        context: format!("Could not get database {mode} settings"),
    })?
    .try_deserialize()
    .map_err(|_| Error::Configuration {
        context: format!("Invalid database {mode} settings"),
    })
}

async fn root_settings() -> Result<DatabaseSettings, Error> {
    database_settings_from_mode("root").await
}

async fn init_root() -> Result<PostgresStorage, Error> {
    let settings = root_settings().await?;
    init_sql_with_prefix(settings, "0").await
}

async fn dev_settings() -> Result<DatabaseSettings, Error> {
    database_settings_from_mode("dev").await
}

async fn init_dev() -> Result<PostgresStorage, Error> {
    let settings = dev_settings().await?;
    init_sql_with_prefix(settings, "1").await
}

async fn init_sql_with_prefix(settings: DatabaseSettings, prefix: &str) -> Result<PostgresStorage, Error> {
    let storage = PostgresStorage::new(settings).await?;
    let sql_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("zero2prod_database")
        .join("sql");
    let mut paths: Vec<PathBuf> = fs::read_dir(sql_dir)
        .context("Could not read sql dir")?
        .filter_map(|entry| {
            let path = entry.ok().map(|e| e.path());
            let name = path
                .clone()
                .map(|p| p.into_os_string())
                .map(|os| os.into_string().ok())
                .flatten();
            // Note here that we filter files starting with '0'. This is the
            // xxx . Files starting with 0 are to be executed as the postgres user.
            if name.map(|s| s.starts_with(prefix)).unwrap_or(false) {
                path
            } else {
                None
            }
        })
        .collect();
    paths.sort();

    for path in paths {
        if let Some(path) = path.to_str() {
            if path.ends_with(".sql") {
                tracing::info!("Executing {path}");
                storage.exec_file(&path).await?;
            }
        }
    }
    Ok(storage)
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
        application::opts::{Command, Opts},
        domain::NewSubscription,
        domain::{SubscriptionStatus, SubscriptionRequest},
        domain::ports::secondary::SubscriptionStorage,
        //domain::ports::secondary::AuthenticationStorage,
    };

    use super::*;

    #[tokio::test]
    async fn storage_should_store_and_retrieve_subscription() {
        let storage = init_dev_db().await.expect("development database");
        let storage = Arc::new(storage);

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
        // Setup & Fixture
        let storage = init_dev_db().await.expect("development database");
        let storage = Arc::new(storage);

        let username = Name().fake::<String>();
        let email = SafeEmail().fake::<String>();
        let request = SubscriptionRequest { username, email };
        let new_subscription = NewSubscription::try_from(request).unwrap();

        let token = 32.fake::<String>();

        // Exec
        let subscription = storage
            .create_subscription_and_store_token(&new_subscription, &token)
            .await
            .expect("storing subscription");

        let id = storage
            .get_subscriber_id_by_token(&token)
            .await
            .expect("getting subscriber id");

        // Check
        assert_that(&id.unwrap()).is_equal_to(subscription.id);
    }

    #[tokio::test]
    async fn storage_should_not_retrieve_subscriber_by_token_once_deleted() {
        // In this test we store a subscription,
        // Then we confirm the subscriber
        // We check that the subscriber's status is 'confirmed'
        // Finally we try to retrieve the subscriber id by the token,
        // which should be deleted from the subscription_token table.
        //
        // Setup & Fixture
        let storage = init_dev_db().await.expect("development database");
        let storage = Arc::new(storage);

        let username = Name().fake::<String>();
        let email = SafeEmail().fake::<String>();
        let request = SubscriptionRequest { username, email };
        let new_subscription = NewSubscription::try_from(request).unwrap();

        let email = new_subscription.email.clone();
        let token = 32.fake::<String>();

        // Exec
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

        // Check
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
