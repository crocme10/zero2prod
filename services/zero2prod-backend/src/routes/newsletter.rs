use axum::extract::{Json, State};
use uuid::Uuid;

use crate::error::ApiError;
use crate::server::AppState;

/// POST handler for newsletter publishing
#[allow(clippy::unused_async)]
#[tracing::instrument(
    name = "Publishing a newsletter issue"
    skip(_state),
    fields(
        request_id = %Uuid::new_v4(),
    )
)]
pub async fn publish_newsletter(State(_state): State<AppState>) -> Result<Json<()>, ApiError> {
    Ok(Json(()))
}
