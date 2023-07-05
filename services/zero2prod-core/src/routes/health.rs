use axum::extract::Json;
use serde::{Deserialize, Serialize};

/// GET handler for health requests by an application platform
///
/// Intended for use in environments such as Amazon ECS or Kubernetes which want
/// to validate that the HTTP service is available for traffic, by returning a
/// 200 OK response with any content.
#[allow(clippy::unused_async)]
pub async fn health() -> Json<Zero2ProdHealthResp> {
    let resp = Zero2ProdHealthResp {
        status: "OK".to_string(),
    };
    Json(resp)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zero2ProdHealthResp {
    pub status: String,
}
