mod error;
pub mod login;
pub mod register;
pub mod static_dir;

pub use self::error::Error;
use self::{
    login::login,
    register::register,
};
use super::AppState;
use axum::routing::{post, Router};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("login", post(login))
        .route("register", post(register))
        .with_state(state)
}
