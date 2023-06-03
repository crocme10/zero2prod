use axum::extract::{Json, State};
use axum_extra::extract::WithRejection;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::server::AppState;

/// POST handler for user subscriptions
#[allow(clippy::unused_async)]
pub async fn subscriptions(
    State(app_state): State<AppState>,
    WithRejection(Json(request), _): WithRejection<Json<SubscriptionRequest>, ApiError>,
) -> Result<Json<Zero2ProdSubscriptionsResp>, ApiError> {
    tracing::info!("request: {:?}", request);
    let SubscriptionRequest { username, email } = request;
    if username.is_empty() {
        return Err(ApiError::new_bad_request("Empty username".to_string()));
    }
    if email.is_empty() {
        return Err(ApiError::new_bad_request("Empty email".to_string()));
    }
    let resp = Zero2ProdSubscriptionsResp {
        status: "OK".to_string(),
    };
    let mut conn = app_state.pool.acquire().await.expect("conn");
    let _ = sqlx::query!(
        r#"INSERT INTO subscriptions (id, email, username, subscribed_at) VALUES ($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        email,
        username,
        Utc::now()
    )
    .execute(&mut conn)
    .await
    .map_err(|err| ApiError::new_internal(format!("Database error: {}", err)))?;
    Ok(Json(resp))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zero2ProdSubscriptionsResp {
    pub status: String,
}

#[derive(Deserialize, Debug)]
pub struct SubscriptionRequest {
    username: String,
    email: String,
}
