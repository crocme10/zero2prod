use clap::Parser;

use zero2prod::server;
use zero2prod::settings;

#[tokio::main]
async fn main() -> Result<(), server::Error> {
    tracing_subscriber::fmt::init();

    let opts = settings::Opts::parse();

    let server = tokio::spawn(server::run(opts));
    if let Err(err) = server.await {
        eprintln!("Error: {err}");
    }

    // match opts.cmd {
    //     settings::Command::Run => { tokio::spawn(server::run(&opts)).await?? },
    //     settings::Command::Config => server::config(&opts).await?,
    // }

    Ok(())
}
