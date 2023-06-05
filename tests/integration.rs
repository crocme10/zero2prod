use cucumber::World;
// use cucumber::WriterExt;
use std::path::PathBuf;

use zero2prod::database::connect_with_conn_str;
use zero2prod::listener::listen_with_host_port;
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

    let settings: Settings = opts.try_into().expect("Compiling Application Settings");

    let conn_str = settings.database.connection_string();

    tracing::info!("Establishing database connection with {}", conn_str);

    let pool = connect_with_conn_str(&conn_str, settings.database.connection_timeout)
        .await
        .unwrap_or_else(|_| panic!("Establishing a database connection with {conn_str}"));

    let listener = listen_with_host_port(settings.network.host.as_str(), settings.network.port)
        .unwrap_or_else(|_| {
            panic!(
                "Could not create listener for {}:{}",
                settings.network.host, settings.network.port
            )
        });

    spawn_service(listener, pool).await;

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
