use common::settings;
use cucumber::when;
use std::path::PathBuf;
use std::time;
use zero2prod::application::opts;

use crate::state::TestWorld;

// Steps are defined with `given`, `when` and `then` attributes.
#[when("the user requests a health check")]
async fn health_check(world: &mut TestWorld) {
    // We have to look into the 'testing' configuration for the port we have to target.
    let opts = opts::Opts {
        config_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("config"),
        run_mode: Some("dev".to_string()),
        settings: vec![],
        cmd: opts::Command::Run,
    };
    let settings: settings::Settings = opts.try_into().expect("Could not get settings");
    let url = format!(
        "{}:{}/api/v1/health",
        settings.application.base_url, settings.application.http
    );

    let client = reqwest::Client::builder()
        .timeout(time::Duration::from_secs(2))
        .build()
        .expect("reqwest client");
    let resp = client.get(url).send().await.expect("health check response");
    world.status_code = Some(resp.status());
}
