use common::config::merge_configuration;
use common::settings::{DatabaseSettings, Settings};
use std::process::Command;

use crate::{check_psql_exists, check_sqlx_exists, project_root};

pub fn db_command() -> Result<(), anyhow::Error> {
    postgres_db()?;
    Ok(())
}

pub fn database_settings() -> DatabaseSettings {
    let config_dir = project_root().join("config");
    println!(
        "Reading database configuration from {}",
        config_dir.display()
    );
    let settings: Settings = merge_configuration(
        &config_dir,
        &["database", "service", "email"],
        "testing",
        "ZERO2PROD",
        vec![],
    )
    .unwrap()
    .try_deserialize()
    .unwrap();
    settings.database
}

pub fn postgres_db() -> Result<(), anyhow::Error> {
    check_psql_exists()?;

    let settings = database_settings();

    println!("Building docker image (zero2prod/database:latest) ...");
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

    println!("Starting docker image (zero2prod/database:latest) ...");
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

    println!("Docker Postgres server online");

    if status.is_err() {
        anyhow::bail!("Could not run docker image");
    }

    println!("Set DATABASE_URL=\"{}\"", settings.connection_string());

    Ok(())
}

pub fn sqlx_prepare() -> Result<(), anyhow::Error> {
    check_sqlx_exists()?;

    let settings = database_settings();

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
