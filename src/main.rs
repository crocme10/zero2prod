use clap::Parser;

use zero2prod::server;
use zero2prod::settings;
use zero2prod::err_context::ErrorContextExt;

#[allow(clippy::result_large_err)]
#[tokio::main]
async fn main() -> Result<(), server::Error> {
    tracing_subscriber::fmt::init();

    let opts = settings::Opts::parse();

    let settings: settings::Settings = opts.try_into().context("Compiling Application Settings".to_string())?;

    let server = tokio::spawn(server::run(settings));
    if let Err(err) = server.await {
        eprintln!("Error: {err}");
    }

    // match opts.cmd {
    //     settings::Command::Run => { tokio::spawn(server::run(&opts)).await?? },
    //     settings::Command::Config => server::config(&opts).await?,
    // }

    Ok(())
}
