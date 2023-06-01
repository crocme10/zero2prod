use serde::{Deserialize, Serialize};
use std::fmt;
use std::{env, path::PathBuf};

use crate::config;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

#[derive(Debug)]
pub enum Error {
    Building {
        context: String,
        source: crate::config::Error,
    },
    Deserializing {
        context: String,
        source: ::config::ConfigError,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Building { context, source } => {
                write!(fmt, "Could not build client request: {context} | {source}")
            }
            Error::Deserializing { context, source } => {
                write!(fmt, "Could not build client request: {context} | {source}")
            }
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, clap::Parser)]
#[clap(
    name = "zero2prod",
    about = "Serving REST API for zero2prod",
    version = VERSION,
    author = AUTHORS
    )]
pub struct Opts {
    /// Defines the config directory
    ///
    #[arg(value_parser = clap::value_parser!(PathBuf), short = 'c', long = "config-dir")]
    pub config_dir: PathBuf,

    /// Defines the run mode in {testing, dev, prod, ...}
    ///
    /// If no run mode is provided, a default behavior will be used.
    // #[arg(short = 'm', long = common::config::ENV_VAR_ENV_NAME.to_lowercase())]
    #[arg(short = 'm', long = "run-mode")]
    pub run_mode: Option<String>,

    /// Override settings values using key=value
    #[arg(short = 's', long = "setting")]
    pub settings: Vec<String>,

    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, clap::Parser)]
pub enum Command {
    /// Execute osm2mimir with the given configuration
    Run,
    /// Prints osm2mimir's configuration
    Config,
}

impl TryInto<Settings> for &Opts {
    type Error = Error;

    fn try_into(self) -> Result<Settings, Self::Error> {
        crate::config::merge_configuration(
            self.config_dir.as_ref(),
            &["service"],
            self.run_mode.as_deref(),
            "NATTER",
            self.settings.clone(),
        )
        .map_err(|err| Error::Building {
            context: "REST Server Settings: Could not merge configuration".to_string(),
            source: err,
        })?
        .try_deserialize()
        .map_err(|err| Error::Deserializing {
            context: "REST Server Settings: Could not deserialize configuration".to_string(),
            source: err,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub network: Network,
    pub mode: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_ok_with_default_config_dir() {
        let opts = Opts {
            config_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config"),
            run_mode: None,
            settings: vec![],
            cmd: Command::Run,
        };

        let settings: Result<Settings, _> = (&opts).try_into();
        println!("settings: {settings:?}");
        assert!(settings.is_ok());
        assert_eq!(settings.unwrap().mode, "default");
    }
}
