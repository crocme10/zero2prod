use cucumber::World;
use reqwest::Response;
use sqlx::{Connection, PgConnection};
use std::path::PathBuf;
use zero2prod::settings::{Command, Opts, Settings};

#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct TestWorld {
    pub db_connection: PgConnection,
    // Wrapped in Option so that TestWorld could derive Default.
    pub resp: Option<Response>,
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

        let conn_string = settings.database.connection_string();

        tracing::info!("Establishing database connection with {}", conn_string);

        let connection = PgConnection::connect(&conn_string)
            .await
            .expect("Failed to connect to Postgres.");

        TestWorld {
            db_connection: connection,
            resp: None,
        }
    }
}
