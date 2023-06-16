use cucumber::writer;
use cucumber::WriterExt;
use cucumber::{writer::Coloring, World as _};
use futures::FutureExt;
use std::sync::Arc;
use tracing_subscriber::{
    filter::LevelFilter,
    fmt::format::{DefaultFields, Format},
    layer::SubscriberExt as _,
    Layer,
};

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
        .fail_on_skipped()
        .with_default_cli()
        .configure_and_init_tracing(
            DefaultFields::new(),
            Format::default().with_ansi(false).without_time(),
            |layer| tracing_subscriber::registry().with(LevelFilter::DEBUG.and_then(layer)),
        )
        .max_concurrent_scenarios(1)
        .with_writer(
            writer::Basic::raw(std::io::stdout(), Coloring::Never, 0)
                .summarized()
                .assert_normalized(),
        )
        .before(move |_feature, _rule, _scenario, world| {
            async {
                let storage = Arc::new(
                    PostgresStorage::new(
                        world.settings.database.clone(),
                        PostgresStorageKind::Testing,
                    )
                    .await
                    .expect("Establishing a database connection"),
                );

                let listener = listen_with_host_port(
                    &world.settings.network.host,
                    world.settings.network.port,
                )
                .expect("Could not create listener");

                let state = State {
                    storage: storage.clone(),
                };
                let server = server::run(listener, state);
                let handle = tokio::spawn(server);
                world.handle = Some(handle);
                world.storage = storage;
            }
            .boxed()
        })
        .after(move |_feature, _rule, _scenario, _event, world| {
            async {
                let handle = world.unwrap().handle.take().expect("handle");
                handle.abort();
            }
            .boxed()
        })
        .run("tests/features")
        .await;
}
