use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

pub mod backend;
pub mod confirmation;
pub mod subscription;

// From yew examples futures
//
// The problem with the initial FetchError implementation (with a single JsValue field)
// is that the JsValue does not hold useful information for the end-user.

// FIXME This is common code, should be moved to zero2prod-common
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct FetchError {
    pub status_code: u16,
    pub description: String,
}

impl Display for FetchError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Fetch Error ({}): {}",
            &self.status_code, &self.description
        )
    }
}

impl Error for FetchError {}

pub enum FetchState<T> {
    NotFetching,
    Fetching,
    Success(T),
    Failed(FetchError),
}
