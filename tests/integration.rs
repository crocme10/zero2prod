use cucumber::World;
use cucumber::WriterExt;
// use sqlx::{postgres::PgConnection, Connection};
use std::path::PathBuf;
use zero2prod::settings::{Command, Opts, Settings};

mod state;
mod steps;
mod utils;

use steps::service::spawn_service;

// This runs before everything else, so you can setup things here.
#[tokio::main]
async fn main() {
    let opts = Opts {
        config_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config"),
        run_mode: Some("testing".to_string()),
        settings: vec![],
        cmd: Command::Run,
    };

    let settings: Settings = opts.try_into().expect("settings");

    tracing::info!("Spawning Service");

    spawn_service(settings.clone()).await;

    // Use the following if we need some debug output
    // state::TestWorld::cucumber()
    //     .max_concurrent_scenarios(1)
    //     .with_writer(
    //         cucumber::writer::Basic::raw(std::io::stdout(), cucumber::writer::Coloring::Never, 0)
    //             .summarized()
    //             .assert_normalized(),
    //     )
    //     .run_and_exit("tests/features")
    //     .await;

    state::TestWorld::cucumber()
        //.init_tracing()
        .run("tests/features")
        .await;
}
