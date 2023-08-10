use axum::http::{header, status::StatusCode};
use axum::response::{IntoResponse, Response};
use axum_extra::extract::cookie::{Cookie, SameSite};
use hyper::header::HeaderMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// GET handler for user logout
#[allow(clippy::unused_async)]
#[tracing::instrument(
    name = "User Logout"
    fields(
        request_id = %Uuid::new_v4(),
    )
)]
pub async fn logout() -> Result<impl IntoResponse, impl IntoResponse> {
    let resp = LogoutResp {
        status: "success".to_string(),
    };

    Ok::<_, ()>(resp)
}

/// This is what we return to the user in response to the logout request.
/// The important stuff happens in the IntoResponse impl, where we kill the cookie.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoutResp {
    pub status: String,
}

impl IntoResponse for LogoutResp {
    fn into_response(self) -> Response {
        let json = serde_json::to_string(&self).unwrap();
        let cookie = Cookie::build("jwt", "")
            .path("/")
            .max_age(time::Duration::hours(-1))
            .same_site(SameSite::Lax)
            .http_only(true)
            .finish();
        let mut headers = HeaderMap::new();
        headers.insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
        (StatusCode::OK, headers, json).into_response()
    }
}
