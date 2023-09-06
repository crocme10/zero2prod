use common::postgres::init_dev_db;
use common::settings::{database_dev_settings, database_root_settings};
use std::process::Command;
use tracing::info;

use crate::{check_psql_exists, check_sqlx_exists, project_root};

pub async fn db_command() -> Result<(), anyhow::Error> {
    postgres_db().await?;
    Ok(())
}

pub async fn postgres_db() -> Result<(), anyhow::Error> {
    check_psql_exists()?;

    info!("Starting postgres docker image with root settings (postgres/15) ...");
    let settings = database_root_settings()
        .await
        .expect("Could not get database root settings");

    let status = Command::new("docker")
        .current_dir(project_root())
        .args([
            "run",
            "--name",
            "zero2prod",
            "-e",
            &format!("POSTGRES_PASSWORD={}", settings.password),
            "-p",
            &format!("{}:5432", settings.port),
            "-d",
            "postgres:15",
        ])
        .status();

    if status.is_err() {
        anyhow::bail!("Could not run docker image");
    }

    wait_for_postgres().await?;

    init_dev_db().await?;

    let settings = database_dev_settings()
        .await
        .expect("Could not get database dev settings");

    info!("Set DATABASE_URL=\"{}\"", settings.connection_string());

    Ok(())
}

pub async fn sqlx_prepare() -> Result<(), anyhow::Error> {
    check_sqlx_exists()?;

    let settings = database_dev_settings()
        .await
        .expect("dev database settings");

    let sqlx_prepare = Command::new("cargo")
        .current_dir(project_root())
        .env("DATABASE_URL", settings.connection_string())
        .args(["sqlx", "prepare", "--workspace"])
        .status();

    if sqlx_prepare.is_err() {
        anyhow::bail!("there was a problem preparing sqlx for offline usage");
    }

    Ok(())
}

async fn wait_for_postgres() -> Result<(), anyhow::Error> {
    let settings = database_root_settings()
        .await
        .expect("Could not get database root settings");

    let mut status = Command::new("psql");
    let status = status
        .current_dir(project_root())
        .env("PGPASSWORD", &settings.password)
        .args([
            "-q",
            "-h",
            "localhost",
            "-U",
            &settings.username,
            "-p",
            &settings.port.to_string(),
            "-d",
            "postgres",
            "-c",
            "\\q",
        ]);

    while !status.status()?.success() {
        println!("Postgres is still unavailable. Waiting to try again...");
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
    Ok(())
}
