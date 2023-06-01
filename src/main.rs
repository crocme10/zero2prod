use clap::Parser;

use zero2prod::server;
use zero2prod::settings;

#[tokio::main]
async fn main() -> Result<(), Box<server::Error>> {
    tracing_subscriber::fmt::init();

    let opts = settings::Opts::parse();

    match opts.cmd {
        settings::Command::Run => server::run(&opts).await?,
        settings::Command::Config => server::config(&opts).await?,
    }

    Ok(())
}
