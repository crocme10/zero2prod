use cucumber::World;
use cucumber::WriterExt;
use cucumber::writer;
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
        .max_concurrent_scenarios(1)
        .with_writer(
            writer::Basic::raw(std::io::stdout(), writer::Coloring::Never, 0)
                .summarized()
                .assert_normalized(),
        )
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

                world.tx = Some(tx);
                let state = State { storage };
                let server = server::run(listener, state, rx);
                let handle = tokio::spawn(server);
                world.handle = Some(handle);
            }
            .boxed()
        })
        .after(move |_feature, _rule, _scenario, _event, world| {
            async {
                // let tx = world.unwrap().tx.take().expect("tx");
                // tx.send(());
                let handle = world.unwrap().handle.take().expect("handle");
                handle.abort();
            }
            .boxed()
        })
        .run("tests/features")
        .await;
}
