use std::env;

use xtask::tasks::ci::ci;
use xtask::tasks::coverage::coverage;
use xtask::tasks::database::{migrate_postgres_db, postgres_db, sqlx_prepare};
use xtask::tasks::frontend::frontend;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(-1);
    }
}

fn try_main() -> Result<(), anyhow::Error> {
    let task = env::args().nth(1);
    match task.as_deref() {
        Some("ci") => ci(),
        Some("coverage") => coverage(),
        Some("frontend") => frontend(),
        // Some("db") => db_command(),
        // Some("dist") => dist(),
        Some("migrate") => migrate_postgres_db(),
        Some("postgres") => postgres_db(),
        // Some("redis") => xtask::tasks::database::setup_redis(),
        Some("prepare") => sqlx_prepare(),
        // Some("test") => xtest(),
        _ => print_help(),
    }
}

fn print_help() -> anyhow::Result<()> {
    eprintln!(
        r#"
Usage: cargo xtask <task>

Tasks:
  test            runs tests on binary and xtask (uses nextest if installed)
  ci              runs all necessary checks to avoid CI errors when git pushed
  coverage        runs test coverage analysis
  dist            builds application and man pages
  prepare         runs the correct sqlx prepare command
  postgres        starts up a postgres docker container and runs migrations
  migrate         runs postgres database migrations
  redis           starts up a redis server
  db              alias for 'postgres' then 'redis'
"#
    );

    Ok(())
}
