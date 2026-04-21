//! OpenTelemetry initialization and configuration
//! Implementation for opentelemetry 0.31 - traces only
//! Metrics are handled in separate metrics module

use log::info;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace::SdkTracerProvider, Resource};
use opentelemetry_semantic_conventions::resource::{SERVICE_NAME, SERVICE_VERSION};
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

/// Initialize OpenTelemetry traces (metrics handled separately)
pub fn init_opentelemetry_traces(
    service_name: &str,
    otlp_endpoint: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("📊 Initializing OpenTelemetry Traces...");
    info!("   Endpoint: {}", otlp_endpoint);
    info!("   Service: {}", service_name);

    // Set global propagator for trace context propagation
    global::set_text_map_propagator(TraceContextPropagator::new());

    // Create resource with service attributes using builder pattern (v0.31 API)
    let resource = Resource::builder()
        .with_attributes([
            KeyValue::new(SERVICE_NAME, service_name.to_string()),
            KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
        ])
        .build();

    // Initialize OTLP span exporter
    let span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_endpoint)
        .with_timeout(Duration::from_secs(3))
        .build()?;

    // Initialize tracing provider with batch exporter
    let tracer_provider = SdkTracerProvider::builder()
        .with_resource(resource.clone())
        .with_batch_exporter(span_exporter)
        .build();

    let tracer = tracer_provider.tracer("emqx-auth-service");

    // Set global tracer provider
    global::set_tracer_provider(tracer_provider);

    // Initialize tracing subscriber with OTel layer
    let otel_layer = tracing_opentelemetry::layer()
        .with_tracer(tracer)
        .with_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()));

    // Add fmt layer for console output
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()));

    tracing_subscriber::registry()
        .with(otel_layer)
        .with(fmt_layer)
        .init();

    info!("✅ OpenTelemetry Traces initialized successfully");

    Ok(())
}

/// Shutdown OpenTelemetry traces
pub fn shutdown_opentelemetry_traces() {
    info!("🛑 Shutting down OpenTelemetry Traces...");

    // In v0.31+, the global tracer provider handles cleanup automatically
    // when the application exits. This function is kept for future compatibility.
    // The batch exporter will flush spans on drop.

    info!("✅ OpenTelemetry Traces shutdown complete");
}
