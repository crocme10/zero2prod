mod error;
pub mod health;
pub mod login;
pub mod logout;
pub mod register;
pub mod static_dir;
pub mod subscriptions;

use super::AppState;
use axum::routing::{get, post, Router};

pub use self::error::Error;
use self::{
    health::health, login::login, logout::logout, register::register, subscriptions::subscriptions,
};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/logout", get(logout))
        .route("/subscriptions", post(subscriptions))
        .with_state(state)
}
