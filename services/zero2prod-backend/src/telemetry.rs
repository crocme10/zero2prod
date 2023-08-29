use crate::application::server::context::Context;
use crate::application::server::routes::Error;
use axum::http::{Method, Uri};
use serde::Serialize;
use serde_json::{json, Value};
use serde_with::skip_serializing_none;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;
use uuid::Uuid;

use tokio::task::JoinHandle;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

/// Sets up a tracing subscriber.
pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let filter_layer =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    let bunyan_format = BunyanFormattingLayer::new(name, sink);

    Registry::default()
        .with(filter_layer)
        .with(JsonStorageLayer)
        .with(bunyan_format)
}

/// Register a subscriber as global default
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    tracing::subscriber::set_global_default(subscriber).unwrap();
}

pub fn spawn_blocking_with_tracing<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || current_span.in_scope(f))
}

pub async fn log_request(
	uuid: Uuid,
	req_method: Method,
	uri: Uri,
	context: Option<Context>,
	error: Option<&Error>,
) {
	let timestamp = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap()
		.as_millis();

	let error_type = error.map(|se| se.to_string());
	let error_data = serde_json::to_value(error)
		.ok()
		.and_then(|mut v| v.get_mut("data").map(|v| v.take()));

	// Create the RequestLogLine
	let log_line = RequestLogLine {
		uuid: uuid.to_string(),
		timestamp: timestamp.to_string(),

		http_path: uri.to_string(),
		http_method: req_method.to_string(),

		user_id: context.map(|c| c.user_id()).flatten(),

		client_error_type: None,

		error_type,
		error_data,
	};

	debug!("REQUEST LOG LINE:\n{}", json!(log_line));
}

#[skip_serializing_none]
#[derive(Serialize)]
struct RequestLogLine {
	uuid: String,      // uuid string formatted
	timestamp: String, // (should be iso8601)

	// -- User and context attributes.
	user_id: Option<Uuid>,

	// -- http request attributes.
	http_path: String,
	http_method: String,

	// -- Errors attributes.
	client_error_type: Option<String>,
	error_type: Option<String>,
	error_data: Option<Value>,
}
