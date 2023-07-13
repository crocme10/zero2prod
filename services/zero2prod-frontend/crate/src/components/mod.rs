// use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use serde::{Serialize, Deserialize};

pub mod backend;
pub mod subscription;
pub mod confirmation;

// From yew examples futures
// The problem with FetchError is that the JsValue does not
// hold useful information for the end-user.

// #[derive(Debug, Clone, PartialEq)]
// pub struct FetchError {
//     err: JsValue,
// }
// 
// impl Display for FetchError {
//     fn fmt(&self, f: &mut Formatter) -> fmt::Result {
//         Debug::fmt(&self.err, f)
//     }
// }
// 
// impl Error for FetchError {}
// 
// impl From<JsValue> for FetchError {
//     fn from(value: JsValue) -> Self {
//         Self { err: value }
//     }
// }
// 
// impl From<serde_wasm_bindgen::Error> for FetchError {
//     fn from(value: serde_wasm_bindgen::Error) -> Self {
//         Self { err: value.into() }
//     }
// }

pub enum FetchState<T> {
    NotFetching,
    Fetching,
    Success(T),
    Failed(Error),
}

// FIXME This is common code, should be moved to zero2prod-common
#[derive(Deserialize, Serialize, Debug)]
pub struct Error {
    pub status_code: u16,
    pub description: String,
}


