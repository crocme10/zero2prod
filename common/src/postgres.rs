use crate::err_context::{ErrorContext, ErrorContextExt};
use crate::settings::{database_dev_settings, database_root_settings, Error as SettingsError};
use serde::Serialize;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::path::{Path, PathBuf};
use std::{fmt, fs};

pub async fn init_dev_db() -> Result<(), Error> {
    tracing::info!("Initializing dev db");
    init_root().await?;
    init_dev().await
}

async fn init_root() -> Result<(), Error> {
    tracing::info!("Initializing database with root SQL files");
    let settings = database_root_settings()
        .await
        .context("Could not get root database settings")?;
    let conn_str = settings.connection_string();
    let root_db = new_db_pool(&conn_str).await?;
    let paths = get_sql_files("0").await?;
    for path in paths {
        exec_file(&root_db, &path).await?;
    }
    Ok(())
}

async fn init_dev() -> Result<(), Error> {
    tracing::info!("Initializing database with dev SQL files");
    let settings = database_dev_settings()
        .await
        .context("Could not get dev database settings")?;
    let conn_str = settings.connection_string();
    let root_db = new_db_pool(&conn_str).await?;
    let paths = get_sql_files("1").await?;
    for path in paths {
        exec_file(&root_db, &path).await?;
    }
    Ok(())
}

async fn get_sql_files(prefix: &str) -> Result<Vec<PathBuf>, Error> {
    let sql_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("services")
        .join("zero2prod-database")
        .join("sql");
    let sql_dir = sql_dir.as_path().canonicalize().context(format!(
        "Could not find cannonical path for {}",
        sql_dir.display()
    ))?;
    let mut paths: Vec<PathBuf> = fs::read_dir(sql_dir.clone())
        .context(format!("Could not read sql dir {}", sql_dir.display()))?
        .filter_map(|entry| {
            let path = entry.ok().map(|e| e.path());
            let name = path
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|f| f.to_str());
            // Note here that we filter files starting with '0'. This is the
            // xxx . Files starting with 0 are to be executed as the postgres user.
            if name
                .map(|s| s.starts_with(prefix) && s.ends_with(".sql"))
                .unwrap_or(false)
            {
                path
            } else {
                None
            }
        })
        .collect();
    paths.sort();
    Ok(paths)
}

async fn new_db_pool(conn_str: &str) -> Result<PgPool, Error> {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(std::time::Duration::from_millis(500))
        .connect(conn_str)
        .await
        .context(format!("Could not establish connection to {conn_str}"))?;

    Ok(pool)
}

async fn exec_file<P: AsRef<Path> + fmt::Debug + ?Sized>(
    db: &PgPool,
    path: &P,
) -> Result<(), Error> {
    let path = path.as_ref();
    let file = path.to_str().ok_or(Error::IO {
        context: format!("Could not get str out of {}", path.display()),
    })?;
    tracing::info!("Executing file {file}");
    let content = fs::read_to_string(file).context("Unable to read file for execution")?;

    // FIXME: Make the split more sql proof.
    let sqls: Vec<&str> = content.split(';').collect();

    for sql in sqls {
        sqlx::query(sql)
            .execute(db)
            .await
            .context("Unable to execute")?;
    }

    Ok(())
}

#[derive(Debug, Serialize)]
pub enum Error {
    /// Error returned by sqlx
    Database {
        context: String,
        source: String,
    },
    Validation {
        context: String,
    },
    /// Connection issue with the database
    Connection {
        context: String,
        source: String,
    },
    Configuration {
        context: String,
        source: SettingsError,
    },
    IO {
        context: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Database { context, source } => {
                write!(fmt, "Database: {context} | {source}")
            }
            Error::Validation { context } => {
                write!(fmt, "Data: {context}")
            }
            Error::Connection { context, source } => {
                write!(fmt, "Database Connection: {context} | {source}")
            }
            Error::Configuration { context, source } => {
                write!(fmt, "Database Configuration: {context} | {source}")
            }
            Error::IO { context } => {
                write!(fmt, "IO Error: {context}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<ErrorContext<sqlx::Error>> for Error {
    fn from(err: ErrorContext<sqlx::Error>) -> Self {
        match err.1 {
            sqlx::Error::PoolTimedOut => Error::Connection {
                context: format!("PostgreSQL Storage: Connection Timeout: {}", err.0),
                source: err.1.to_string(),
            },
            sqlx::Error::Database(_) => Error::Database {
                context: format!("PostgreSQL Storage: Database: {}", err.0),
                source: err.1.to_string(),
            },
            _ => Error::Connection {
                context: format!(
                    "PostgreSQL Storage: Could not establish a connection: {}",
                    err.0
                ),
                source: err.1.to_string(),
            },
        }
    }
}

impl From<ErrorContext<SettingsError>> for Error {
    fn from(err: ErrorContext<SettingsError>) -> Self {
        Error::Configuration {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<std::io::Error>> for Error {
    fn from(err: ErrorContext<std::io::Error>) -> Self {
        Error::IO {
            context: format!("{}: {}", err.0, err.1),
        }
    }
}
