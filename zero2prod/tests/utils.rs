use std::path::PathBuf;
use zero2prod_common::settings;
use zero2prod::opts;

pub fn testing_url_for_endpoint(endpoint: &str) -> String {
    // We have to look into the 'testing' configuration for the port we have to target.
    let opts = opts::Opts {
        config_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config"),
        run_mode: Some("testing".to_string()),
        settings: vec![],
        cmd: opts::Command::Run,
    };
    let settings: settings::Settings = opts.try_into().expect("Could not get settings");
    format!(
        "http://{}:{}/{}",
        settings.network.host, settings.network.port, endpoint
    )
}
