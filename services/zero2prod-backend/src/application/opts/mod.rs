pub mod cli;
mod error;

pub use self::error::Error;
pub use cli::{Command, Opts};
