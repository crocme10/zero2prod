use cucumber::cli;
use cucumber::writer;
use cucumber::WriterExt;
use cucumber::{writer::Coloring, World as _};
use futures::FutureExt;
use tracing_subscriber::{
    filter::LevelFilter,
    fmt::format::{DefaultFields, Format},
    layer::SubscriberExt as _,
    Layer,
};

mod state;
mod steps;

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
        .after(move |_feature, _rule, _scenario, _event, world| {
            async {
                let app = world.and_then(|w| w.app.take());

                if let Some(mut app) = app {
                    tracing::info!("Aborting server");
                    if let Some(handle) = app.server_handle.take() {
                        handle.abort();
                    }
                    tracing::info!("Dropping Test App");
                    drop(app); // Make sure we drop the app, which drops the listener
                }
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
