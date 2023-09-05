/// Implementation of authentication_store and subscriptions_store using postgres
mod authentication;
mod error;
mod subscription;

pub use self::error::Error;

use common::err_context::ErrorContextExt;
use common::settings::{database_dev_settings, DatabaseSettings};
use sqlx::PgConnection;
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use std::path::{Path, PathBuf};
use std::{fmt, fs};

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

    #[tracing::instrument(name = "Executing SQL file", skip(self))]
    pub async fn exec_file<P: AsRef<Path> + fmt::Debug + ?Sized>(
        &self,
        path: &P,
    ) -> Result<(), Error> {
        let path = path.as_ref();
        let file = path.to_str().ok_or(Error::IO {
            context: format!("Could not get str out of {}", path.display()),
        })?;
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

// FIXME Getting mixed up with root and dev.
// This function will be called by init_dev_db, to drop and recreate
// the database. Since the database should have been created with the
// `cargo xtask postgres` command, I end up using the dev profile here too.
async fn init_root() -> Result<(), Error> {
    tracing::info!("Initializing database with root SQL files");
    let paths = get_sql_files("0").await?;
    for path in paths {
        let settings = database_dev_settings()
            .await
            .context("Could not get root database settings")?;
        let conn_str = settings.connection_string();
        let root_db = new_db_pool(&conn_str).await?;
        exec_file(&root_db, &path).await?;
    }
    Ok(())
}

async fn init_dev() -> Result<PostgresStorage, Error> {
    let settings = database_dev_settings()
        .await
        .context("Could not get dev database settings")?;
    init_sql_with_prefix(settings, "1").await
}

async fn init_sql_with_prefix(
    settings: DatabaseSettings,
    prefix: &str,
) -> Result<PostgresStorage, Error> {
    tracing::info!("Initializing database with SQL files prefixed with {prefix}");
    let paths = get_sql_files(prefix).await?;
    let storage = PostgresStorage::new(settings).await?;
    for path in paths {
        if let Some(path) = path.to_str() {
            if path.ends_with(".sql") {
                storage.exec_file(path).await?;
            }
        }
    }
    Ok(storage)
}

async fn get_sql_files(prefix: &str) -> Result<Vec<PathBuf>, Error> {
    let sql_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("zero2prod-database")
        .join("sql");
    let sql_dir = sql_dir.as_path().canonicalize().context(format!(
        "Could not find cannonical path for {}",
        sql_dir.display()
    ))?;
    let mut paths: Vec<PathBuf> = fs::read_dir(sql_dir.clone())
        .context(format!("Could not read sql dir {}", sql_dir.display()))?
        .filter_map(|entry| {
            let path = entry.ok().map(|e| e.path());
            let name = path
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|f| f.to_str());
            // Note here that we filter files starting with '0'. This is the
            // xxx . Files starting with 0 are to be executed as the postgres user.
            if name
                .map(|s| s.starts_with(prefix) && s.ends_with(".sql"))
                .unwrap_or(false)
            {
                path
            } else {
                None
            }
        })
        .collect();
    paths.sort();
    Ok(paths)
}

async fn new_db_pool(conn_str: &str) -> Result<PgPool, Error> {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(std::time::Duration::from_millis(500))
        .connect(conn_str)
        .await
        .context(format!(
            "Could not establish connection to {conn_str}"
        ))?;

    Ok(pool)
}

async fn exec_file<P: AsRef<Path> + fmt::Debug + ?Sized>(
    db: &PgPool,
    path: &P,
) -> Result<(), Error> {
    let path = path.as_ref();
    let file = path.to_str().ok_or(Error::IO {
        context: format!("Could not get str out of {}", path.display()),
    })?;
    tracing::info!("{:<12} - pexec: {file}", "FOR-DEV-ONLY");
    let content = fs::read_to_string(file).context("Unable to read file for execution")?;

    // FIXME: Make the split more sql proof.
    let sqls: Vec<&str> = content.split(';').collect();

    for sql in sqls {
        sqlx::query(sql)
            .execute(db)
            .await
            .context("Unable to execute")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::name::en::Name;
    use fake::Fake;
    use speculoos::prelude::*;
    use std::sync::Arc;

    use crate::{
        domain::ports::secondary::SubscriptionStorage,
        //domain::ports::secondary::AuthenticationStorage,
        domain::NewSubscription,
        domain::{SubscriptionRequest, SubscriptionStatus},
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
