use config::{Config, ConfigError, Environment, File};
use std::{env, fmt, path::Path};
use tracing::trace;

use crate::err_context::{ErrorContext, ErrorContextExt};

static DEFAULT_ENV_NAME: &str = "default";
static LOCAL_ENV_NAME: &str = "local";

#[derive(Debug)]
pub enum Error {
    Configuration {
        context: &'static str,
        source: ConfigError,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Configuration { context, source } => {
                write!(
                    fmt,
                    "Could not create configuration: {context} | source: {source}"
                )
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<ErrorContext<&'static str, ConfigError>> for Error {
    fn from(context: ErrorContext<&'static str, ConfigError>) -> Error {
        Error::Configuration {
            context: context.0,
            source: context.1,
        }
    }
}

pub fn merge_configuration<
    'a,
    R: Into<Option<&'a str>> + Clone,
    P: Into<Option<&'a str>>,
    D: AsRef<str>,
>(
    root_dir: &Path,
    sub_dirs: &[D],
    profile: R,
    prefix: P,
    overrides: Vec<String>,
) -> Result<Config, Error> {
    let mut builder = sub_dirs
        .iter()
        .try_fold(Config::builder(), |mut builder, sub_dir| {
            let dir_path = root_dir.join(sub_dir.as_ref());

            // First we read the default configuration.
            let default_path = dir_path.join(DEFAULT_ENV_NAME);

            trace!(
                "Reading default configuration from: {}",
                default_path.display()
            );

            builder = builder.add_source(File::from(default_path));

            // Then we look at the profile. If a profile was defined using enviranment variable,
            // we merge that one. Otherwise, if we have one as a function argument (probably coming
            // from the CLI) then we use that. If no run mode is given, we don't do anything.
            if let Some(profile) = env::var("ZERO2PROD_PROFILE")
                .ok()
                .or_else(|| profile.clone().into().map(String::from))
            {
                let profile_path = dir_path.join(profile);

                trace!(
                    "Reading profile configuration from: {}",
                    profile_path.display()
                );

                builder = builder.add_source(File::from(profile_path).required(false));
            }

            // Add in a local configuration file
            // This file shouldn't be checked in to git
            let local_path = dir_path.join(LOCAL_ENV_NAME);

            trace!("Reading local configuration from: {}", local_path.display());

            builder = builder.add_source(File::from(local_path).required(false));

            Ok::<_, Error>(builder)
        })?;

    if let Some(prefix) = prefix.into() {
        let prefix = Environment::with_prefix(prefix).separator("_");
        builder = builder.add_source(prefix)
    }

    // Add command line overrides
    if !overrides.is_empty() {
        builder = builder.add_source(config_from_args(overrides)?)
    }

    builder
        .build()
        .context("Could not merge configuration")
        .map_err(|err| err.into())
}

// Create a new configuration source from a list of assignments key=value
fn config_from_args(args: impl IntoIterator<Item = String>) -> Result<Config, Error> {
    let builder = args.into_iter().fold(Config::builder(), |builder, arg| {
        builder.add_source(File::from_str(&arg, config::FileFormat::Toml))
    });
    builder
        .build()
        .context("Could not build configuration from args")
        .map_err(|err| err.into())
}

#[cfg(test)]
mod tests {
    // Note: We must serialize tests as some tests depend on environment variables.
    use super::*;

    #[test]
    fn should_correctly_create_a_source_from_int_assignment() {
        let overrides = vec![String::from("foo=42")];
        let config = config_from_args(overrides).unwrap();
        let val: i32 = config.get("foo").unwrap();
        assert_eq!(val, 42);
    }

    #[test]
    #[should_panic(expected = "unexpected eof")]
    fn should_correctly_report_an_error_on_invalid_assignment() {
        let overrides = vec![String::from("foo=")];
        let config = config_from_args(overrides).unwrap();
        let val: i32 = config.get("foo").unwrap();
        assert_eq!(val, 42);
    }

    #[test]
    fn should_correctly_create_a_source_from_string_assignment() {
        let overrides = vec![String::from("foo='42'")];
        let config = config_from_args(overrides).unwrap();
        let val: String = config.get("foo").unwrap();
        assert_eq!(val, "42");
    }

    #[test]
    fn should_correctly_create_a_source_from_array_assignment() {
        let overrides = vec![String::from("foo=['fr', 'en']")];
        let config = config_from_args(overrides).unwrap();
        let val: Vec<String> = config.get("foo").unwrap();
        assert_eq!(val[0], "fr");
        assert_eq!(val[1], "en");
    }

    #[test]
    fn should_correctly_create_a_source_from_multiple_assignments() {
        let overrides = vec![
            String::from("database.url='http://localhost:5432'"),
            String::from("service.port=6666"),
        ];
        let config = config_from_args(overrides).unwrap();
        let url: String = config.get("database.url").unwrap();
        let port: i32 = config.get("service.port").unwrap();
        assert_eq!(url, "http://localhost:5432");
        assert_eq!(port, 6666);
    }
}
