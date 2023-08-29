use axum::handler::HandlerWithoutStateExt;
use axum::http::StatusCode;
use axum::routing::{any_service, MethodRouter};
use std::path::PathBuf;
use tower_http::services::ServeDir;

pub fn static_dir(static_dir: PathBuf) -> MethodRouter {
    async fn handle_404() -> (StatusCode, &'static str) {
        (StatusCode::NOT_FOUND, "Resource not found.")
    }

    any_service(ServeDir::new(static_dir).not_found_service(handle_404.into_service()))
}
