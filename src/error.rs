// inspired by https://shanmukhsista.com/posts/technology/apis/error-handling-in-rest-apis-using-axum-framework-and-custom-error-model-rest-apis/
//
use axum::http::status::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use std::fmt;

#[derive(Serialize, Debug)]
pub struct ApiError {
    pub status_code: u16,
    pub errors: Vec<String>,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("Err {} ", &self.status_code))
    }
}

impl ApiError {
    pub fn new(status_code: u16, err: String) -> Self {
        let errors: Vec<String> = vec![err];
        ApiError {
            status_code,
            errors,
        }
    }

    pub fn new_internal(err: String) -> Self {
        let errors: Vec<String> = vec![err];
        ApiError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            errors,
        }
    }

    pub fn new_bad_request(err: String) -> Self {
        let errors: Vec<String> = vec![err];
        ApiError {
            status_code: StatusCode::BAD_REQUEST.as_u16(),
            errors,
        }
    }

    pub fn new_unauthorized(err: String) -> Self {
        let errors: Vec<String> = vec![err];
        ApiError {
            status_code: StatusCode::UNAUTHORIZED.as_u16(),
            errors,
        }
    }

    pub fn new_missing_auth_header(err: String) -> Self {
        let errors: Vec<String> = vec![err];
        ApiError {
            status_code: 700, // Our code, not a common HTTP Code
            errors,
        }
    }

    pub fn append_error(&mut self, err: String) {
        let _ = &self.errors.push(err);
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
