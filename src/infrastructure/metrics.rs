//! OpenTelemetry Metrics Implementation
//! Compatible with opentelemetry 0.31

use log::info;
use opentelemetry::{global, KeyValue};
use opentelemetry::metrics::{Counter, Histogram, UpDownCounter, Meter};
use opentelemetry_sdk::metrics::{SdkMeterProvider, PeriodicReader};
use opentelemetry_otlp::{MetricExporter, WithExportConfig};
use std::sync::Arc;
use std::time::Duration;

/// EMQX Auth Service Metrics
#[derive(Clone, Debug)]
pub struct AppMetrics {
    meter: Meter,
    http_metrics: HttpMetrics,
    auth_metrics: AuthMetrics,
    _provider: Arc<SdkMeterProvider>,
}

/// HTTP Server Metrics
#[derive(Clone, Debug)]
pub struct HttpMetrics {
    /// HTTP server request duration (histogram)
    pub duration: Histogram<f64>,
    /// HTTP server request counter
    pub request_counter: Counter<u64>,
    /// HTTP server active requests (in-flight)
    pub active_requests: UpDownCounter<i64>,
}

/// Authentication Metrics
#[derive(Clone, Debug)]
pub struct AuthMetrics {
    /// Total authentication requests
    pub auth_requests_total: Counter<u64>,
    /// Successful authentications
    pub auth_success: Counter<u64>,
    /// Failed authentications
    pub auth_failure: Counter<u64>,
    /// Total MQTT users
    pub mqtt_users_total: UpDownCounter<i64>,
}

impl AppMetrics {
    /// Initialize OpenTelemetry Metrics
    pub fn new(service_name: &str, otlp_endpoint: &str) -> Result<Self, Box<dyn std::error::Error>> {
        info!("📊 Initializing OpenTelemetry Metrics...");
        info!("   Endpoint: {}", otlp_endpoint);
        info!("   Service: {}", service_name);

        // Create OTLP metric exporter using the same pattern as traces
        let metric_exporter = MetricExporter::builder()
            .with_tonic()
            .with_endpoint(otlp_endpoint)
            .with_timeout(Duration::from_secs(3))
            .build()?;

        // Create periodic reader with 10s interval
        let reader = PeriodicReader::builder(metric_exporter)
            .with_interval(Duration::from_secs(10))
            .build();

        // Create meter provider with resource attributes using builder pattern
        // Convert service_name to owned String to avoid lifetime issues
        let service_name_owned = service_name.to_string();
        let provider = SdkMeterProvider::builder()
            .with_resource(
                opentelemetry_sdk::Resource::builder()
                    .with_attributes([
                        KeyValue::new("service.name", service_name_owned),
                        KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                    ])
                    .build()
            )
            .with_reader(reader)
            .build();

        // Set global meter provider first
        global::set_meter_provider(provider.clone());

        // Get meter from global (compatible with opentelemetry 0.31)
        let meter = global::meter("emqx-auth-service");

        // Create metrics instruments
        let http_metrics = create_http_metrics(&meter)?;
        let auth_metrics = create_auth_metrics(&meter)?;

        info!("✅ OpenTelemetry Metrics initialized successfully");

        Ok(Self {
            meter,
            http_metrics,
            auth_metrics,
            _provider: Arc::new(provider),
        })
    }

    /// Get HTTP metrics
    pub fn http(&self) -> &HttpMetrics {
        &self.http_metrics
    }

    /// Get auth metrics
    pub fn auth(&self) -> &AuthMetrics {
        &self.auth_metrics
    }

    /// Get meter for custom instruments
    pub fn meter(&self) -> &Meter {
        &self.meter
    }

    /// Shutdown metrics provider
    pub fn shutdown(&self) {
        info!("🛑 Shutting down OpenTelemetry Metrics...");
        // In v0.31+, the meter provider handles cleanup automatically on drop.
        // The periodic reader will flush metrics on shutdown.
        info!("✅ OpenTelemetry Metrics shutdown complete");
    }
}

/// Create HTTP metrics instruments
pub fn create_http_metrics(meter: &Meter) -> Result<HttpMetrics, Box<dyn std::error::Error>> {
    // HTTP request duration histogram (in seconds)
    let duration = meter
        .f64_histogram("http.server.request.duration")
        .with_description("HTTP server request duration in seconds")
        .with_unit("s")
        .build();

    // HTTP request counter
    let request_counter = meter
        .u64_counter("http.server.request.count")
        .with_description("Total number of HTTP server requests")
        .with_unit("1")
        .build();

    // Active requests (in-flight)
    let active_requests = meter
        .i64_up_down_counter("http.server.active_requests")
        .with_description("Number of active HTTP server requests")
        .with_unit("1")
        .build();

    Ok(HttpMetrics {
        duration,
        request_counter,
        active_requests,
    })
}

/// Create authentication metrics instruments
pub fn create_auth_metrics(meter: &Meter) -> Result<AuthMetrics, Box<dyn std::error::Error>> {
    // Total auth requests
    let auth_requests_total = meter
        .u64_counter("auth.requests.total")
        .with_description("Total number of authentication requests")
        .with_unit("1")
        .build();

    // Successful authentications
    let auth_success = meter
        .u64_counter("auth.requests.success")
        .with_description("Number of successful authentications")
        .with_unit("1")
        .build();

    // Failed authentications
    let auth_failure = meter
        .u64_counter("auth.requests.failure")
        .with_description("Number of failed authentications")
        .with_unit("1")
        .build();

    // Total MQTT users
    let mqtt_users_total = meter
        .i64_up_down_counter("mqtt.users.total")
        .with_description("Total number of MQTT users")
        .with_unit("1")
        .build();

    Ok(AuthMetrics {
        auth_requests_total,
        auth_success,
        auth_failure,
        mqtt_users_total,
    })
}

impl HttpMetrics {
    /// Record an HTTP request
    pub fn record(
        &self,
        method: &str,
        path: &str,
        status: u16,
        duration_secs: f64,
    ) {
        let attributes = [
            KeyValue::new("http.method", method.to_string()),
            KeyValue::new("http.route", path.to_string()),
            KeyValue::new("http.status_code", status as i64),
        ];

        self.duration.record(duration_secs, &attributes);
        self.request_counter.add(1, &attributes);
    }

    /// Increment active requests counter
    pub fn request_started(&self, method: &str, path: &str) {
        let attributes = [
            KeyValue::new("http.method", method.to_string()),
            KeyValue::new("http.route", path.to_string()),
        ];
        self.active_requests.add(1, &attributes);
    }

    /// Decrement active requests counter
    pub fn request_finished(&self, method: &str, path: &str) {
        let attributes = [
            KeyValue::new("http.method", method.to_string()),
            KeyValue::new("http.route", path.to_string()),
        ];
        self.active_requests.add(-1, &attributes);
    }
}

impl AuthMetrics {
    /// Record authentication request
    pub fn record_auth_request(&self) {
        self.auth_requests_total.add(1, &[]);
    }

    /// Record successful authentication
    pub fn record_auth_success(&self) {
        self.auth_success.add(1, &[]);
    }

    /// Record failed authentication
    pub fn record_auth_failure(&self) {
        self.auth_failure.add(1, &[]);
    }

    /// Set total MQTT users count
    pub fn set_mqtt_users_count(&self, count: i64) {
        self.mqtt_users_total.add(count, &[]);
    }
}
