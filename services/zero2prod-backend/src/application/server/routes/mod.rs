mod error;
pub mod health;
pub mod login;
pub mod logout;
pub mod register;
pub mod static_dir;
pub mod subscription_confirmation;
pub mod subscriptions;
pub mod newsletter;

use super::AppState;
use axum::routing::{get, post, Router};

pub use self::error::Error;
use self::{
    health::health, login::login, logout::logout, register::register,
    subscription_confirmation::subscriptions_confirmation, subscriptions::subscriptions,
    newsletter::publish_newsletter,
};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/logout", get(logout))
        .route("/subscriptions", post(subscriptions))
        .route(
            "/subscription_confirmation",
            post(subscriptions_confirmation),
        )
        .route("/newsletter/publish", post(publish_newsletter))
        .with_state(state)
}
