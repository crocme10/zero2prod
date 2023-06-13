use cucumber::World;
use futures::FutureExt;
use std::sync::Arc;

mod state;
mod steps;
mod utils;

use zero2prod::listener::listen_with_host_port;
use zero2prod::postgres::{PostgresStorage, PostgresStorageKind};
use zero2prod::server::{self, State};

// This runs before everything else, so you can setup things here.
#[tokio::main]
async fn main() {
    state::TestWorld::cucumber()
        .before(move |_feature, _rule, _scenario, world| {
            async {
                let storage = Arc::new(
                    PostgresStorage::new(world.settings.database.clone(), PostgresStorageKind::Testing)
                        .await
                        .expect("Establishing a database connection"),
                );

                let listener = listen_with_host_port(
                    &world.settings.network.host,
                    world.settings.network.port,
                )
                .expect("Could not create listener");

                let (tx, rx) = tokio::sync::oneshot::channel::<()>();

                world.tx = tx;
                let state = State { storage };
                let server = server::run(listener, state, rx);
                let server = tokio::spawn(server);
                if let Err(err) = server.await {
                    eprintln!("Error: {err}");
                }
            }
            .boxed()
        })
        .after(move |_feature, _rule, _scenario, _event, world| {
            async {
                world.unwrap().tx.send(());
            }
            .boxed()
        })
        .run("tests/features")
        .await;
}
