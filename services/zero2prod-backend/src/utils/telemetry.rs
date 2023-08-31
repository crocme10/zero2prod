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
    global::set_text_map_propagator(TraceContextPropagator::new());

    global::set_error_handler(|error| error!(error = format!("{error:#}"), "otel error"))
        .expect("set error handler");

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(otlp_layer(settings))
        .try_init()
        .expect("initialize tracing subscriber")
}

/// Create an OTLP layer exporting tracing data.
fn otlp_layer<S>(settings: TracingSettings) -> impl Layer<S>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    let exporter = opentelemetry_otlp::new_exporter()
        .http()
        .with_endpoint(settings.otlp_exporter_endpoint);

    let trace_config = trace::config().with_resource(Resource::new(vec![KeyValue::new(
        "service.name",
        settings.service_name,
    )]));

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(trace_config)
        .install_batch(runtime::Tokio)
        .expect("install tracer");

    tracing_opentelemetry::layer().with_tracer(tracer)
}
