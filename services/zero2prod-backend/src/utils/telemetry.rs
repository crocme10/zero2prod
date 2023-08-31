use tokio::task::JoinHandle;
use axum::http::request::Request;
use axum::body::Body;
use common::settings::TracingSettings;
use opentelemetry::{
    global, runtime,
    sdk::{propagation::TraceContextPropagator, trace, Resource},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use tracing::{error, Subscriber};
use tracing::Span;
use tracing::info_span;
use tracing_subscriber::{
    layer::SubscriberExt, registry::LookupSpan, util::SubscriberInitExt, EnvFilter, Layer,
};

pub fn spawn_blocking_with_tracing<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || current_span.in_scope(f))
}

pub fn make_span(request: &Request<Body>) -> Span {
    let headers = request.headers();
    info_span!("incoming request", ?headers)
}

/// Initialize tracing: apply an `EnvFilter` using the `RUST_LOG` environment variable to define the
/// log levels, add a formatter layer logging trace events as JSON and on OpenTelemetry layer
/// exporting trace data.
pub fn init_tracing(settings: TracingSettings) {

    let TracingSettings { service_name, otlp_exporter_endpoint, level } = settings;

    global::set_text_map_propagator(TraceContextPropagator::new());

    global::set_error_handler(|error| error!(error = format!("{error:#}"), "otel error"))
        .expect("set error handler");

    let filter_layer =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(otlp_layer(otlp_exporter_endpoint, service_name))
        .try_init()
        .expect("initialize tracing subscriber")
}

/// Create an OTLP layer exporting tracing data.
fn otlp_layer<S>(otlp_exporter_endpoint: String, service_name: String) -> impl Layer<S>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    let exporter = opentelemetry_otlp::new_exporter()
        .http()
        .with_endpoint(otlp_exporter_endpoint);

    let trace_config = trace::config().with_resource(Resource::new(vec![KeyValue::new(
        "service.name",
        service_name,
    )]));

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(trace_config)
        .install_batch(runtime::Tokio)
        .expect("install tracer");

    tracing_opentelemetry::layer().with_tracer(tracer)
}
