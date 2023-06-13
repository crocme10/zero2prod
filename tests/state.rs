use cucumber::World;
use reqwest::Response;
// use sqlx::{PgPool, Postgres, Transaction};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::oneshot;

use zero2prod::postgres::{PostgresStorage, PostgresStorageKind};
use zero2prod::settings::{Command, Opts, Settings};

#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct TestWorld {
    pub settings: Settings,
    pub storage: Arc<PostgresStorage>,
    pub resp: Option<Response>,
    pub tx: oneshot::Sender<()>,
}

impl TestWorld {
    pub async fn new() -> Self {
        let opts = Opts {
            config_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config"),
            run_mode: Some("testing".to_string()),
            settings: vec![],
            cmd: Command::Run,
        };

        let settings: Settings = opts.try_into().expect("settings");

        let storage = Arc::new(
            PostgresStorage::new(settings.database.clone(), PostgresStorageKind::Testing)
            .await
            .expect("Storage connection")
            );

        // We create a channel now, but it will be replaced during the test before hook.
        let (tx, _) = tokio::sync::oneshot::channel::<()>();

        TestWorld {
            settings,
            storage,
            resp: None,
            tx,
        }
    }
}
