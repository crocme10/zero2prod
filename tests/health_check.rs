use cucumber::{given, then, when, World, WriterExt};
use reqwest::{Response, StatusCode};
use zero2prod::{settings, server};
use std::path::PathBuf;

#[derive(Debug, Default, World)]
pub struct TestWorld {
    // Wrapped in Option so that TestWorld could derive Default.
    resp: Option<Response>,
}

#[given("the service has been started")]
async fn start_service(_world: &mut TestWorld) {
    spawn_service().await
}

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
    let url = format!("http://{}:{}/health", settings.network.host, settings.network.port);
    let resp = reqwest::get(url)
        .await
        .expect("response");
    world.resp = Some(resp);
}

#[then("the response is 200 OK")]
fn response_is_ok(world: &mut TestWorld) {
    assert!(world.resp.as_ref().unwrap().status() == StatusCode::OK);
}

async fn spawn_service() {
    let opts = settings::Opts {
        config_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config"),
        run_mode: Some("testing".to_string()),
        settings: vec![],
        cmd: settings::Command::Run,
    };

    let _ = tokio::spawn(server::run(opts));
}

// This runs before everything else, so you can setup things here.
#[tokio::main]
async fn main() {
    TestWorld::cucumber()
        .max_concurrent_scenarios(1)
        .with_writer(
            cucumber::writer::Basic::raw(std::io::stdout(), cucumber::writer::Coloring::Never, 0)
                .summarized()
                .assert_normalized(),
        )
        .run_and_exit("tests/features")
        .await;
}
