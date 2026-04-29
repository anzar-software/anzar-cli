use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{EnvFilter, Registry, fmt::MakeWriter, layer::SubscriberExt};

use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use tracing_opentelemetry::OpenTelemetryLayer;

pub fn get_subscriber<Sink>(
    name: &str,
    env_filter: &str,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    let formatting_layer = BunyanFormattingLayer::new(name.into(), sink);

    // 1. Build the OTLP exporter
    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_protocol(opentelemetry_otlp::Protocol::HttpBinary)
        .with_endpoint("http://localhost:4318/v1/traces")
        .build()
        .expect("Failed to build OTLP span exporter");

    // 2. Build the tracer provider
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(otlp_exporter)
        .with_resource(Resource::builder().with_service_name("anzar").build())
        .build();

    // 3. Register globally and get a tracer
    global::set_tracer_provider(tracer_provider.clone());
    let tracer = tracer_provider.tracer("anzar");

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
        .with(OpenTelemetryLayer::new(tracer))
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // LogTracer::init().expect("Failed to set logger");
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}
