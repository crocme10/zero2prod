// inspired by https://shanmukhsista.com/posts/technology/apis/error-handling-in-rest-apis-using-axum-framework-and-custom-error-model-rest-apis/
//
use axum::http::status::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::extract::rejection::JsonRejection;
use serde::Serialize;
use std::fmt;

#[derive(Serialize, Debug)]
pub struct ApiError {
    pub status_code: u16,
    pub description: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("Err {} ", &self.status_code))
    }
}

impl ApiError {
    pub fn new(status_code: u16, err: String) -> Self {
        ApiError {
            status_code,
            description: err,
        }
    }

    pub fn new_internal(err: String) -> Self {
        tracing::error!("Internal Server Error: {}", err);
        ApiError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            description: err,
        }
    }

    pub fn new_bad_request(err: String) -> Self {
        tracing::error!("Bad Request Error: {}", err);
        ApiError {
            status_code: StatusCode::BAD_REQUEST.as_u16(),
            description: err,
        }
    }

    pub fn new_unauthorized(err: String) -> Self {
        tracing::error!("Unauthorized Error: {}", err);
        ApiError {
            status_code: StatusCode::UNAUTHORIZED.as_u16(),
            description: err,
        }
    }

    pub fn new_missing_auth_header(err: String) -> Self {
        ApiError {
            status_code: 700, // Our code, not a common HTTP Code
            description: err,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            StatusCode::from_u16(self.status_code).unwrap(),
            serde_json::to_string(&self).unwrap(),
        )
            .into_response()
    }
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> ApiError {
        match rejection {
            JsonRejection::JsonDataError(err) => {
                serde_json_error_response(err)
            }
            JsonRejection::JsonSyntaxError(err) => {
                serde_json_error_response(err)
            }
            // handle other rejections from the `Json` extractor
            JsonRejection::MissingJsonContentType(_) => 
                ApiError::new_bad_request( "Missing `Content-Type: application/json` header".to_string()),
            JsonRejection::BytesRejection(_) =>
                ApiError::new_internal( "Failed to buffer request body".to_string()),
            // we must provide a catch-all case since `JsonRejection` is marked
            // `#[non_exhaustive]`
            _ => ApiError::new_internal( "Unknown error".to_string()),
        }
    }
}
// From the axum description on extractor: https://docs.rs/axum/latest/axum/extract/index.html
// attempt to extract the inner `serde_json::Error`, if that succeeds we can
// provide a more specific error
fn serde_json_error_response<E>(err: E) -> ApiError
where
    E: std::error::Error + 'static,
{
    if let Some(serde_json_err) = find_error_source::<serde_json::Error>(&err) {
        ApiError::new_bad_request (
            format!(
                "Invalid JSON at line {} column {}",
                serde_json_err.line(),
                serde_json_err.column()
            ),
        )
    } else {
        ApiError::new_bad_request ( "Unknown error".to_string())
    }
}

// attempt to downcast `err` into a `T` and if that fails recursively try and
// downcast `err`'s source
fn find_error_source<'a, T>(err: &'a (dyn std::error::Error + 'static)) -> Option<&'a T>
where
    T: std::error::Error + 'static,
{
    if let Some(err) = err.downcast_ref::<T>() {
        Some(err)
    } else if let Some(source) = err.source() {
        find_error_source(source)
    } else {
        None
    }
}
