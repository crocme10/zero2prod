use cucumber::given;
use std::net::TcpListener;
use sqlx::postgres::PgPool;

use zero2prod::{server};

use crate::state;

#[given("the service has been started")]
async fn start_service(_world: &mut state::TestWorld) {
    // println!("Starting service");
    // spawn_service().await
}

pub async fn spawn_service (listener: TcpListener, pool: PgPool) {
    let _ = tokio::spawn(server::run(listener, pool));
}
