use cucumber::cli;
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

use zero2prod::email_service_impl::EmailClient;
use zero2prod::listener::listen_with_host_port;
use zero2prod::postgres::PostgresStorage;
use zero2prod::server;
// TODO See how to use telemetry within integration tests.

#[derive(cli::Args)] // re-export of `clap::Args`
struct CustomOpts {
    /// Use tracing for debug output
    #[arg(long)]
    trace: Option<bool>,
}

#[tokio::main]
async fn main() {
    let opts = cli::Opts::<_, _, _, CustomOpts>::parsed();
    let trace = opts.custom.trace.unwrap_or_default();

    let cucumber = state::TestWorld::cucumber()
        .fail_on_skipped()
        .with_cli(opts)
        .max_concurrent_scenarios(1)
        .before(move |_feature, _rule, _scenario, world| {
            async {
                // We should be using the same code as in main, namely use
                // Application::new(settings). But we need access to the storage, so that
                // transaction can be dropped (and aborted) at the end of each scenario.
                // So the following code is essentially what's happening inside the
                // Application::new function.
                let storage = Arc::new(
                    PostgresStorage::new(world.settings.database.clone())
                        .await
                        .expect("Establishing a database connection"),
                );

                let email = Arc::new(
                    EmailClient::new(world.settings.email_client.clone())
                        .await
                        .expect("Establishing an email service connection"),
                );

                let listener = listen_with_host_port(
                    world.settings.network.host.as_str(),
                    world.settings.network.port,
                )
                .expect(&format!(
                    "Could not create listener for {}:{}",
                    world.settings.network.host, world.settings.network.port
                ));

                let server = server::new(
                    listener,
                    storage.clone(),
                    email,
                    world.settings.network.host.clone(),
                );
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
        });

    if trace {
        cucumber
            .configure_and_init_tracing(
                DefaultFields::new(),
                Format::default().with_ansi(false).without_time(),
                |layer| tracing_subscriber::registry().with(LevelFilter::DEBUG.and_then(layer)),
                // { telemetry::get_subscriber("test".to_string(), "debug".to_string()).with(layer) }
            )
            .with_writer(
                writer::Basic::raw(std::io::stdout(), Coloring::Never, 0)
                    .summarized()
                    .assert_normalized(),
            )
            .run("tests/features")
            .await;
    } else {
        cucumber.run("tests/features").await;
    }
}
