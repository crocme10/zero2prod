mod error;
pub mod login;
pub mod static_dir;

pub use self::error::Error;
use self::login::login;
use super::AppState;
use axum::routing::{post, Router};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/login", post(login))
        .with_state(state)
}
