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
        .before(move |_feature, _rule, _scenario, world| {
            async {
                // Abort the server if its running, and restart the app
                if let Some(handle) = world.app.server_handle.take() {
                    handle.abort();
                }
                world.app = state::spawn_app().await;
                world.subscribers.clear();
                world.users.clear();
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
