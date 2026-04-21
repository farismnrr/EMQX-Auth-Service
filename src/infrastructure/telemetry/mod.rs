//! Telemetry module - OpenTelemetry metrics and tracing

pub mod metrics;
pub mod telemetry;

pub use metrics::AppMetrics;
pub use telemetry::{init_opentelemetry_traces, shutdown_opentelemetry_traces};
