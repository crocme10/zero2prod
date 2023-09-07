/// Implementation of authentication_store and subscriptions_store using postgres
mod authentication;
mod error;
mod subscription;

pub use self::error::Error;

use common::err_context::ErrorContextExt;
use common::settings::DatabaseSettings;
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};

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

#[cfg(test)]
mod tests {
    use common::postgres::init_dev_db;
    use common::settings::database_dev_settings;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::name::en::Name;
    use fake::Fake;
    use serial_test::serial;
    use speculoos::prelude::*;
    use std::sync::Arc;

    use crate::{
        domain::ports::secondary::SubscriptionStorage,
        //domain::ports::secondary::AuthenticationStorage,
        domain::NewSubscription,
        domain::{SubscriptionRequest, SubscriptionStatus},
    };

    use super::*;

    #[serial]
    #[tokio::test]
    async fn storage_should_store_and_retrieve_subscription() {
        init_dev_db()
            .await
            .expect("Could not reinitialization development database");
        let settings = database_dev_settings()
            .await
            .expect("Could not retrieve development database settings");
        let storage = Arc::new(
            PostgresStorage::new(settings)
                .await
                .expect("Could not get pool for development database"),
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

    #[serial]
    #[tokio::test]
    async fn storage_should_store_and_retrieve_subscriber_by_token() {
        init_dev_db()
            .await
            .expect("Could not reinitialization development database");
        let settings = database_dev_settings()
            .await
            .expect("Could not retrieve development database settings");
        let storage = Arc::new(
            PostgresStorage::new(settings)
                .await
                .expect("Could not get pool for development database"),
        );
        // Setup & Fixture

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

    #[serial]
    #[tokio::test]
    async fn storage_should_not_retrieve_subscriber_by_token_once_deleted() {
        // In this test we store a subscription,
        // Then we confirm the subscriber
        // We check that the subscriber's status is 'confirmed'
        // Finally we try to retrieve the subscriber id by the token,
        // which should be deleted from the subscription_token table.
        //
        // Setup & Fixture
        init_dev_db()
            .await
            .expect("Could not reinitialization development database");
        let settings = database_dev_settings()
            .await
            .expect("Could not retrieve development database settings");
        let storage = Arc::new(
            PostgresStorage::new(settings)
                .await
                .expect("Could not get pool for development database"),
        );

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
