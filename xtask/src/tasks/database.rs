use common::settings::{database_root_settings, database_dev_settings};
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
        .expect("Could not get database dev settings");

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
