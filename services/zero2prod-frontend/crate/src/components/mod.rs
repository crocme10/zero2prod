use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use wasm_bindgen::JsValue;

pub mod backend;
pub mod subscription;

// From yew examples futures

#[derive(Debug, Clone, PartialEq)]
pub struct FetchError {
    err: JsValue,
}

impl Display for FetchError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(&self.err, f)
    }
}

impl Error for FetchError {}

impl From<JsValue> for FetchError {
    fn from(value: JsValue) -> Self {
        Self { err: value }
    }
}

impl From<serde_wasm_bindgen::Error> for FetchError {
    fn from(value: serde_wasm_bindgen::Error) -> Self {
        Self { err: value.into() }
    }
}

pub enum FetchState<T> {
    NotFetching,
    Fetching,
    Success(T),
    Failed(FetchError),
}
