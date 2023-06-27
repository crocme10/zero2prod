use clap::Parser;
use std::fmt;

use zero2prod::application::{Application, Error as ApplicationError};
use zero2prod::opts::{Command, Error as OptsError, Opts};
use zero2prod::telemetry;
use zero2prod_common::err_context::{ErrorContext, ErrorContextExt};
use zero2prod_common::settings::{Error as SettingsError, Settings};

#[derive(Debug)]
pub enum Error {
    Options {
        context: String,
        source: OptsError,
    },
    Application {
        context: String,
        source: ApplicationError,
    },
    Configuration {
        context: String,
        source: SettingsError,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Configuration { context, source } => {
                write!(
                    fmt,
                    "REST Server: Could not build configuration: {context} | {source}"
                )
            }
            Error::Application { context, source } => {
                write!(fmt, "Could not build application: {context} | {source}")
            }
            Error::Options { context, source } => {
                write!(fmt, "Options Error: {context} | {source}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<ErrorContext<String, SettingsError>> for Error {
    fn from(err: ErrorContext<String, SettingsError>) -> Self {
        Error::Configuration {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<String, ApplicationError>> for Error {
    fn from(err: ErrorContext<String, ApplicationError>) -> Self {
        Error::Application {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<String, OptsError>> for Error {
    fn from(err: ErrorContext<String, OptsError>) -> Self {
        Error::Options {
            context: err.0,
            source: err.1,
        }
    }
}

#[allow(clippy::result_large_err)]
#[tokio::main]
async fn main() -> Result<(), Error> {
    let subscriber =
        telemetry::get_subscriber("zero2prod".to_string(), "info".to_string(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let opts = Opts::parse();

    let cmd = opts.cmd.clone();

    let settings: Settings = opts
        .try_into()
        .context("Compiling Application Settings".to_string())?;

    match cmd {
        Command::Config => {
            println!("{}", serde_json::to_string_pretty(&settings).unwrap());
        }
        Command::Run => {
            let app = Application::new(settings)
                .await
                .context("could not build application".to_string())?;
            app.run_until_stopped()
                .await
                .context("application runtime error".to_string())?;
        }
    }
    Ok(())
}
