use std::{env, process::Command, thread, time::Duration};
use zero2prod_common::config::merge_configuration;
use zero2prod_common::settings::{DatabaseSettings, Settings};

use crate::{check_psql_exists, check_sqlx_exists, project_root};

pub fn db_command() -> Result<(), anyhow::Error> {
    postgres_db()?;
    // setup_redis()?;
    Ok(())
}

pub fn sqlx_prepare() -> Result<(), anyhow::Error> {
    wait_for_postgres()?;
    check_sqlx_exists()?;

    let settings = database_settings();

    let sqlx_prepare = Command::new("cargo")
        .current_dir(project_root().join("zero2prod"))
        .env("DATABASE_URL", settings.connection_string())
        .args(["sqlx", "prepare"])
        .status();

    let mv_sqlx_data = Command::new("mv")
        .current_dir(project_root())
        .args(["zero2prod/sqlx-data.json", "."])
        .status();

    if sqlx_prepare.is_err() || mv_sqlx_data.is_err() {
        anyhow::bail!("there was a problem running preparing sqlx-data.json");
    }

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

    let skip_docker = env::var("SKIP_DOCKER")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    if skip_docker {
        println!("Skipping docker...");
    } else {
        println!("Starting docker image...");
        let mut _status = Command::new("docker")
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
                "postgres",
                "postgres",
                "-N",
                "1000",
            ])
            .status()?;

        wait_for_postgres()?;

        println!("Docker Postgres server online");
    }

    // Migrate the database automatically as part of initialization
    migrate_postgres_db()?;

    println!("Set DATABASE_URL=\"{}\"", settings.connection_string());

    Ok(())
}

pub fn migrate_postgres_db() -> Result<(), anyhow::Error> {
    wait_for_postgres()?;
    check_sqlx_exists()?;

    let settings = database_settings();

    println!("Migrating database...");

    println!("DATABASE_URL: {}", settings.connection_string());
    let migration_status1 = Command::new("sqlx")
        .current_dir(project_root())
        .env("DATABASE_URL", settings.connection_string())
        .args(["database", "create"])
        .status();

    let migration_status2 = Command::new("sqlx")
        .current_dir(project_root().join("zero2prod"))
        .env("DATABASE_URL", settings.connection_string())
        .args(["migrate", "run"])
        .status();

    if migration_status1.is_err() || migration_status2.is_err() {
        anyhow::bail!("there was a problem running the migration");
    }

    println!("Postgres migration completed.");

    Ok(())
}

fn wait_for_postgres() -> Result<(), anyhow::Error> {
    let settings = database_settings();

    // TODO: If we're checking that postgres is available before
    // we run the migration, then we can't target the 'settings.database_name', but
    // rather we should target 'postgres'. But 'postgres' is not available in
    // remote DO environment it seems.
    // => Maybe check if the host is 'localhost', then target postgres,
    // otherwise target settings.database_name
    let mut check_online = Command::new("psql");
    let check_online = check_online
        .current_dir(project_root())
        .env("PGPASSWORD", settings.password)
        .args([
            "-h",
            &settings.host,
            "-U",
            &settings.username,
            "-p",
            &settings.port.to_string(),
            "-d",
            &settings.database_name,
            //"postgres",
            "-c",
            "\\q",
        ]);

    while !check_online.status()?.success() {
        println!("Postgres is still unavailable. Waiting to try again...");
        thread::sleep(Duration::from_millis(1000));
    }
    Ok(())
}

// pub fn setup_redis() -> Result<(), anyhow::Error> {
//     let running_container = Command::new("docker")
//         .args([
//             "ps",
//             "--filter",
//             "name=zero2prod_redis",
//             "--format",
//             "{{.ID}}",
//         ])
//         .output()
//         .unwrap();
//     let running_container_id = String::from_utf8(running_container.stdout).unwrap();
//     let running_container_id = running_container_id.trim().to_string();
//
//     if !running_container_id.is_empty() {
//         anyhow::bail!(
//             "Redis container already running.\n\t\
//             Use `docker rm -f {}` to stop and destroy it.",
//             running_container_id
//         );
//     }
//
//     Command::new("docker")
//         .current_dir(project_root())
//         .args([
//             "run",
//             "-p",
//             "6379:6379",
//             "-d",
//             "--name",
//             format!("zero2prod_redis_{}", chrono::Local::now().format("%s")).as_str(),
//             "redis:7",
//         ])
//         .status()?;
//     println!("Redis done");
//     Ok(())
// }
