use crate::telemetry::log_request;
use crate::application::server::routes::Error;
use axum::http::{Method, Uri};
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::application::server::context::Context;

pub async fn error(
	context: Option<Context>,
	uri: Uri,
	method: Method,
	resp: Response,
) -> Response {
	tracing::debug!("{:<12} - mw_reponse_map", "RES_MAPPER");

	let uuid = Uuid::new_v4();

	// -- Get the eventual response error.
	let error = resp.extensions().get::<Error>();
	let code_and_json = error.map(|err| err.standardize());

	// -- If client error, build the new reponse.
	let error_resp =
		code_and_json
			.as_ref()
			.map(|(status_code, value)| {
				(*status_code, value.clone()).into_response()
			});

	let _ = log_request(
		uuid,
		method,
		uri,
		// context,
        context,
		error,
	)
	.await;

	error_resp.unwrap_or(resp)
}
