use cucumber::{given, then, when, World, WriterExt};
use reqwest::{Response, StatusCode};
use tokio::time::{sleep, Duration};
use zero2prod::{settings, server};
use std::path::PathBuf;

#[derive(Debug, Default, World)]
pub struct TestWorld {
    // Wrapped in Option so that TestWorld could derive Default.
    resp: Option<Response>,
}

#[given("the service has been started")]
async fn start_service(_world: &mut TestWorld) {
    println!("Starting service");
    spawn_service().await
}

// Steps are defined with `given`, `when` and `then` attributes.
#[when("the user requests a health check")]
async fn health_check(world: &mut TestWorld) {
    let resp = reqwest::get("http://127.0.0.1:3000/health_check")
        .await
        .expect("response");
    world.resp = Some(resp);
}

#[then("the response is 200 OK")]
fn response_is_ok(world: &mut TestWorld) {
    assert!(world.resp.as_ref().unwrap().status() == StatusCode::OK);
}

async fn spawn_service() {
    sleep(Duration::from_millis(100)).await;
    println!("100 ms have elapsed");
    // dbg!("Starting service");
    // println!("Starting service");
    let opts = settings::Opts {
        config_dir: PathBuf::from(r"./config"),
        run_mode: Some("testing".to_string()),
        settings: vec![],
        cmd: settings::Command::Run,
    };

    if let Err(err) = server::run(&opts).await {
        eprintln!("zero2prod service error: {}", err);
    }
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
