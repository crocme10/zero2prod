use cucumber::given;
use zero2prod::{server, settings};

use crate::state;

#[given("the service has been started")]
async fn start_service(_world: &mut state::TestWorld) {
    // println!("Starting service");
    // spawn_service().await
}

pub async fn spawn_service(settings: settings::Settings) {
    let _ = tokio::spawn(server::run(settings));
}
