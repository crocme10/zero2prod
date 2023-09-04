use crate::settings::TracingSettings;
use opentelemetry::{
    global,
    sdk::{propagation::TraceContextPropagator, trace},
};
use tracing::error;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter};

/// Initialize tracing: apply an `EnvFilter` using the `RUST_LOG` environment variable to define the
/// log levels, add a formatter layer logging trace events as JSON and on OpenTelemetry layer
/// exporting trace data.
pub fn init_tracing(settings: TracingSettings) {
    let TracingSettings { level, jaeger } = settings;

    global::set_text_map_propagator(TraceContextPropagator::new());

    global::set_error_handler(|error| error!(error = format!("{error:#}"), "otel error"))
        .expect("set error handler");

    let filter_layer = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    let subscriber = tracing_subscriber::Registry::default()
        .with(filter_layer)
        .with(fmt::Layer::new().with_writer(std::io::stdout));

    if let Some(jaeger) = jaeger {
        let tracer = jaeger_tracer(&jaeger.endpoint, &jaeger.service_name);
        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
        let subscriber = subscriber.with(telemetry);
        tracing::subscriber::set_global_default(subscriber).unwrap();
    } else {
        tracing::subscriber::set_global_default(subscriber).unwrap();
    }
}

fn jaeger_tracer(endpoint: &str, service_name: &str) -> trace::Tracer {
    opentelemetry_jaeger::new_agent_pipeline()
        .with_endpoint(endpoint)
        .with_service_name(service_name)
        .install_simple()
        .expect("jaeger tracer")
}
