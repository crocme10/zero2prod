use cucumber::World;
use reqwest::Response;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::task::JoinHandle;

use hyper::Error;
use zero2prod::opts::{Command, Opts};
use zero2prod::postgres::PostgresStorage;
use zero2prod_common::settings::Settings;

/// The TestWorld contains both the context for every tests,
/// and information that needs to be kept between steps of a
/// scenario.
#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct TestWorld {
    pub settings: Settings,
    /// TODO: Should we keep 'PostgresStorage', or should
    /// we use '<dyn Storage>' ?
    pub storage: Arc<PostgresStorage>,
    pub resp: Option<Response>,
    pub handle: Option<JoinHandle<Result<(), Error>>>,
}

impl TestWorld {
    /// Creates a new TestWorld, using a 'testing' configuration.
    pub async fn new() -> Self {
        let opts = Opts {
            config_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("config"),
            run_mode: Some("testing".to_string()),
            settings: vec![],
            cmd: Command::Run,
        };

        let settings: Settings = opts.try_into().expect("settings");

        let storage = Arc::new(
            PostgresStorage::new(settings.database.clone())
                .await
                .expect("Storage connection"),
        );

        TestWorld {
            settings,
            storage,
            resp: None,
            handle: None,
        }
    }
}
