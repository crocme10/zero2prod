use cucumber::given;
use std::path::PathBuf;
use zero2prod::{server, settings};

use crate::state;

#[given("the service has been started")]
async fn start_service(_world: &mut state::TestWorld) {
    // println!("Starting service");
    // spawn_service().await
}

pub async fn spawn_service() {
    let opts = settings::Opts {
        config_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config"),
        run_mode: Some("testing".to_string()),
        settings: vec![],
        cmd: settings::Command::Run,
    };

    let _ = tokio::spawn(server::run(opts));
}
