use cucumber::World;
use reqwest::Response;
use sqlx::{PgPool, Postgres, Transaction};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::oneshot::Sender;
use tokio::sync::Mutex;

use zero2prod::database::connect_with_conn_str;
use zero2prod::settings::{Command, Opts, Settings};
use zero2prod::server::State;

#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct TestWorld {
    pub pool: PgPool,
    pub host: String,
    pub port: u16,
    pub exec: Option<Arc<Mutex<State<Transaction<'static, Postgres>>>>>,
    pub resp: Option<Response>,
    pub tx: Option<Sender<()>>,
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

        let conn_str = settings.database.connection_string();

        let pool = connect_with_conn_str(&conn_str, settings.database.connection_timeout)
            .await
            .unwrap_or_else(|_| panic!("Establishing a database connection with {conn_str}"));


        TestWorld {
            pool,
            host: settings.network.host,
            port: settings.network.port,
            exec: None,
            resp: None,
            tx: None,
        }
    }
}
