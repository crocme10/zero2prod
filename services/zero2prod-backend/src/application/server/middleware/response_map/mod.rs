use crate::application::server::routes::Error;
use axum::response::{IntoResponse, Response};

pub async fn error(resp: Response) -> Response {
    tracing::debug!("{:<12} - mw_reponse_map", "RES_MAPPER");

    // -- Get the eventual response error.
    let error = resp.extensions().get::<Error>();
    let code_and_json = error.map(|err| err.standardize());

    // -- If client error, build the new reponse.
    let error_resp = code_and_json
        .as_ref()
        .map(|(status_code, value)| (*status_code, value.clone()).into_response());

    error_resp.unwrap_or(resp)
}
