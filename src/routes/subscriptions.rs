use axum::extract::Json;
use axum_extra::extract::WithRejection;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

/// POST handler for user subscriptions
#[allow(clippy::unused_async)]
pub async fn subscriptions(
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
