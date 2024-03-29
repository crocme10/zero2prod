use common::config;
use common::settings;
use std::{env, path::PathBuf};

use super::Error;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

#[derive(Debug, Clone, clap::Parser)]
#[clap(
    name = "zero2prod",
    about = "Serving REST API for zero2prod",
    version = VERSION,
    author = AUTHORS
    )]
pub struct Opts {
    /// Defines the config directory
    #[arg(value_parser = clap::value_parser!(PathBuf), short = 'c', long = "config-dir")]
    pub config_dir: PathBuf,

    /// Defines the run mode in {testing, dev, prod, ...}
    ///
    /// If no run mode is provided, the default behavior will be used.
    #[arg(short = 'm', long = "run-mode")]
    pub run_mode: Option<String>,

    /// Override settings values using key=value
    #[arg(short = 's', long = "setting")]
    pub settings: Vec<String>,

    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, Clone, clap::Parser)]
pub enum Command {
    /// Execute zero2prod with the given configuration
    Run,
    /// Prints zero2prod configuration
    Config,
}

impl TryInto<settings::Settings> for Opts {
    type Error = Error;

    fn try_into(self) -> Result<settings::Settings, Self::Error> {
        config::merge_configuration(
            self.config_dir.as_ref(),
            &["service", "database", "email", "tracing"],
            self.run_mode.as_deref(),
            "ZERO2PROD",
            self.settings.clone(),
        )
        .map_err(|err| Error::Merging {
            context: "Zero2Prod Server Settings: Could not merge configuration".to_string(),
            source: err,
        })?
        .try_deserialize()
        .map_err(|err| Error::Deserializing {
            context: "Zero2Prod Server Settings: Could not deserialize configuration".to_string(),
            source: err,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_ok_with_default_config_dir() {
        let opts = Opts {
            config_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
                .join("config"),
            run_mode: None,
            settings: vec![],
            cmd: Command::Run,
        };

        let settings: Result<settings::Settings, _> = opts.try_into();
        assert!(settings.is_ok());
        assert_eq!(settings.unwrap().mode, "default");
    }
}
