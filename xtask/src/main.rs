use std::env;

use common::tracing;
use xtask::tasks::certificate::certificate;
use xtask::tasks::ci::ci;
use xtask::tasks::coverage::coverage;
use xtask::tasks::database::{postgres_db, sqlx_prepare};
use xtask::tasks::frontend::frontend;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let settings = common::settings::tracing_dev_settings()
        .await
        .expect("tracing dev settings");
    tracing::init_tracing(settings);
    try_main().await
}

async fn try_main() -> Result<(), anyhow::Error> {
    let task = env::args().nth(1);
    match task.as_deref() {
        Some("ci") => ci(),
        Some("coverage") => coverage(),
        Some("frontend") => frontend(),
        Some("certificate") => certificate(),
        Some("postgres") => postgres_db().await,
        Some("prepare") => sqlx_prepare().await,
        _ => print_help(),
    }
}

fn print_help() -> anyhow::Result<()> {
    eprintln!(
        r#"
Usage: cargo xtask <task>

Tasks:
  test            runs tests on binary and xtask (uses nextest if installed)
  certificate     generate certificates for TLS
  ci              runs all necessary checks to avoid CI errors when git pushed
  coverage        runs test coverage analysis
  dist            builds application and man pages
  frontend        builds frontend
  postgres        starts up a postgres docker container and create the database therein
  prepare         sqlx prepare
"#
    );

    Ok(())
}
