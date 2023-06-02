use cucumber::when;
use std::path::PathBuf;
use zero2prod::settings;

use crate::state::TestWorld;

// Steps are defined with `given`, `when` and `then` attributes.
#[when("the user requests a health check")]
async fn health_check(world: &mut TestWorld) {
    // We have to look into the 'testing' configuration for the port we have to target.
    let opts = settings::Opts {
        config_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config"),
        run_mode: Some("testing".to_string()),
        settings: vec![],
        cmd: settings::Command::Run,
    };
    let settings: settings::Settings = opts.try_into().expect("Could not get settings");
    let url = format!(
        "http://{}:{}/health",
        settings.network.host, settings.network.port
    );
    let resp = reqwest::get(url).await.expect("response");
    world.resp = Some(resp);
}
