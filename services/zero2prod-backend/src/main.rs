use clap::Parser;
use std::fmt;

use common::err_context::{ErrorContext, ErrorContextExt};
use common::settings::Settings;
use zero2prod::application::opts::{Command, Error as OptsError, Opts};
use zero2prod::application::{ApplicationBuilder, Error as ApplicationError};
use zero2prod::utils::telemetry;

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
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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

impl From<ErrorContext<ApplicationError>> for Error {
    fn from(err: ErrorContext<ApplicationError>) -> Self {
        Error::Application {
            context: err.0,
            source: err.1,
        }
    }
}

impl From<ErrorContext<OptsError>> for Error {
    fn from(err: ErrorContext<OptsError>) -> Self {
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

    let settings: Settings = opts.try_into().context("Compiling Application Settings")?;

    match cmd {
        Command::Config => {
            println!("{}", serde_json::to_string_pretty(&settings).unwrap());
        }
        Command::Run => {
            let app = ApplicationBuilder::new(settings)
                .await
                .context("could not build application")?
                .build();
            app.run_until_stopped()
                .await
                .context("application runtime error")?;
        }
    }
    Ok(())
}
