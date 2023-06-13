use cucumber::World;
use reqwest::Response;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use zero2prod::postgres::{PostgresStorage, PostgresStorageKind};
use zero2prod::settings::{Command, Opts, Settings};
use zero2prod::server::Error;

#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct TestWorld {
    pub settings: Settings,
    pub storage: Arc<PostgresStorage>,
    pub resp: Option<Response>,
    pub tx: Option<oneshot::Sender<()>>,
    pub handle: Option<JoinHandle<Result<(), Error>>>
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

        TestWorld {
            settings,
            storage,
            resp: None,
            tx: None,
            handle: None,
        }
    }
}
