mod error;
pub use self::error::Error;

use config::{Config, Environment, File};
use std::{env, path::Path};
use tracing::trace;

use crate::err_context::ErrorContextExt;

static DEFAULT_ENV_NAME: &str = "default";
static LOCAL_ENV_NAME: &str = "local";

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
        let prefix = Environment::with_prefix(prefix)
            .prefix_separator("__")
            .separator("__");
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
    use serial_test::serial;
    use std::path::PathBuf;

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

    #[test]
    #[serial]
    fn should_correctly_read_default_from_directory() {
        // In this test, we expect the function 'merge_configuration' to pick the default
        // configuration in the given subdirectory. Of course this relies on the fact that
        // the environment variable is not set. So, in the test, we back it up, unset it,
        // and then restore it.
        let back_profile_var = env::var_os("ZERO2PROD_PROFILE");
        // Here we create a scope, which will trigger the scopeguard on exit.
        {
            // We create a scopeguard to restore the previous value of the environment variable,
            // because we unset it during this test, to make sure the environment during the test
            // is without the variable set.
            scopeguard::defer! {
                // If the variable was set, restore it, otherwise do nothing.
                if let Some(value) = back_profile_var {
                    env::set_var("ZERO2PROD_PROFILE", value);
                }
            }
            env::remove_var("ZERO2PROD_PROFILE");
            let mut root_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            root_path.push("tests/resources/config");
            let config = merge_configuration(&root_path, &["service"], None, None, vec![]).unwrap();
            let value: String = config.get("identity.username").unwrap();
            assert_eq!(value, "foo");
        }
    }

    #[test]
    #[serial]
    fn should_correctly_overwrite_with_arg_mode() {
        // In this test, we call merge_configuration with a 'dev' mode. There is both a default and
        // a dev file, with the same key. The final configuration should have the dev value.
        let back_profile_var = env::var_os("ZERO2PROD_PROFILE");
        // Here we create a scope, which will trigger the scopeguard on exit.
        {
            // We create a scopeguard to restore the previous value of the environment variable,
            // because we unset it during this test, to make sure the environment during the test
            // is without the variable set.
            scopeguard::defer! {
                // If the variable was set, restore it, otherwise do nothing.
                if let Some(value) = back_profile_var {
                    env::set_var("ZERO2PROD_PROFILE", value);
                }
            }
            env::remove_var("ZERO2PROD_PROFILE");
            let mut root_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            root_path.push("tests/resources/config");
            let config =
                merge_configuration(&root_path, &["service"], "dev", None, vec![]).unwrap();
            let value: String = config.get("identity.username").unwrap();
            assert_eq!(value, "bar");
        }
    }

    #[test]
    #[serial]
    fn should_correctly_overwrite_with_env_mode() {
        // In this test, we call merge_configuration with a 'prod' mode set using an env var. There is both a default and
        // a prod file, with the same key. The final configuration should have the dev value.
        let back_profile_var = env::var_os("ZERO2PROD_PROFILE");
        // Here we create a scope, which will trigger the scopeguard on exit.
        {
            // We create a scopeguard to restore the previous value of the environment variable,
            // because we unset it during this test, to make sure the environment during the test
            // is without the variable set.
            scopeguard::defer! {
                // If the variable was set, restore it, otherwise do nothing.
                if let Some(value) = back_profile_var {
                    env::set_var("ZERO2PROD_PROFILE", value);
                }
            }
            env::set_var("ZERO2PROD_PROFILE", "prod");
            let mut root_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            root_path.push("tests/resources/config");
            let config = merge_configuration(&root_path, &["service"], None, None, vec![]).unwrap();
            let value: String = config.get("identity.username").unwrap();
            assert_eq!(value, "baz");
        }
    }

    #[test]
    #[serial]
    fn should_correctly_ignore_unknown_mode() {
        // In this test, if there is no file corresponding to the given mode, then we use the
        // default. So we specify a 'cloud' environment, for which there is no configuration.
        let back_profile_var = env::var_os("ZERO2PROD_PROFILE");
        // Here we create a scope, which will trigger the scopeguard on exit.
        {
            // We create a scopeguard to restore the previous value of the environment variable,
            // because we unset it during this test, to make sure the environment during the test
            // is without the variable set.
            scopeguard::defer! {
                // If the variable was set, restore it, otherwise do nothing.
                if let Some(value) = back_profile_var {
                    env::set_var("ZERO2PROD_PROFILE", value);
                }
            }
            env::remove_var("ZERO2PROD_PROFILE");
            let mut root_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            root_path.push("tests/resources/config");
            let config =
                merge_configuration(&root_path, &["service"], "cloud", None, vec![]).unwrap();
            let value: String = config.get("identity.username").unwrap();
            assert_eq!(value, "foo");
        }
    }

    #[test]
    #[should_panic(expected = "not found")]
    fn should_correctly_report_an_error_when_missing_default() {
        // In this test, there should be no 'service' directory under the root directory.
        // Yet, if the user specifies this 'service' directory, he should be warned.
        let mut root_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        root_path.push("tests/resources/config");
        let config = merge_configuration(&root_path, &["service"], None, None, vec![]).unwrap();
        let port: i32 = config.get("service.port").unwrap();
        assert_eq!(port, 5432);
    }

    #[test]
    #[serial]
    fn should_correctly_overwrite_with_values() {
        // In this test, we call merge_configuration with a 'prod' mode set using an argument. But
        // then we overwrite the 'value'.
        let back_profile_var = env::var_os("ZERO2PROD_PROFILE");
        // Here we create a scope, which will trigger the scopeguard on exit.
        {
            scopeguard::defer! {
                // If the variable was set, restore it, otherwise do nothing.
                if let Some(value) = back_profile_var {
                    env::set_var("ZERO2PROD_PROFILE", value);
                }
            }
            env::set_var("ZERO2PROD_PROFILE", "prod");
            let mut root_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            root_path.push("tests/resources/config");
            let config = merge_configuration(
                &root_path,
                &["service"],
                None,
                None,
                vec![String::from("identity.username = 'zot'")],
            )
            .unwrap();
            let value: String = config.get("identity.username").unwrap();
            assert_eq!(value, "zot");
        }
    }
}
