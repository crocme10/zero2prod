use common::settings::database_dev_settings;
use std::process::Command;
use tracing::info;

use crate::{check_psql_exists, check_sqlx_exists, project_root};

pub async fn db_command() -> Result<(), anyhow::Error> {
    postgres_db().await?;
    Ok(())
}

pub async fn postgres_db() -> Result<(), anyhow::Error> {
    check_psql_exists()?;

    info!("Building docker image (zero2prod/database:latest) ...");
    let status = Command::new("docker")
        .current_dir(project_root())
        .args([
            "buildx",
            "build",
            "--load",
            "--tag",
            "zero2prod/database:latest",
            "-f",
            "services/zero2prod-database/Dockerfile",
            "services/zero2prod-database",
        ])
        .status();

    if status.is_err() {
        anyhow::bail!("Could not build docker image");
    }

    info!("Starting docker image with dev settings (zero2prod/database:latest) ...");
    let settings = database_dev_settings()
        .await
        .expect("Could not get database dev settings");

    let status = Command::new("docker")
        .current_dir(project_root())
        .args([
            "run",
            "--name",
            "zero2prod",
            "-e",
            &format!("POSTGRES_USER={}", settings.username),
            "-e",
            &format!("POSTGRES_PASSWORD={}", settings.password),
            "-e",
            &format!("POSTGRES_DB={}", settings.database_name),
            "-p",
            &format!("{}:5432", settings.port),
            "-d",
            "zero2prod/database:latest",
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
