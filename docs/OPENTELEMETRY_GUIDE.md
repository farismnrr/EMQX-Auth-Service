# 📊 OpenTelemetry Integration Guide

**EMQX Auth Service** now uses **OpenTelemetry v0.31** for standardized logging, tracing, and metrics.

---

## 🎯 OVERVIEW

The service now implements the **OpenTelemetry standard v0.31** for:
- ✅ **Structured Logging** - JSON-formatted logs with trace context
- ✅ **Distributed Tracing** - End-to-end request tracing
- ✅ **Metrics** - Performance and error metrics (stub implementation)

All telemetry data is exported via **OTLP (OpenTelemetry Protocol)** to your observability backend.

**Last Updated:** March 2026
**OpenTelemetry Version:** 0.31.0
**tracing-opentelemetry Version:** 0.32.1

---

## 🚀 QUICK START

### 1. **Set Environment Variables**

```bash
# Required: OTLP endpoint (default: http://localhost:4317)
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317

# Required: Service name (default: emqx-auth-service)
OTEL_SERVICE_NAME=emqx-auth-service

# Optional: Additional resource attributes
OTEL_RESOURCE_ATTRIBUTES=deployment.environment=production,service.version=1.0.0

# Optional: Log level (default: info)
RUST_LOG=info,actix_web=info,sea_orm=warn
```

### 2. **Run the Service**

```bash
cargo run
```

You should see:
```
✅ OpenTelemetry initialized successfully
📝 Logging configured with level: info
```

---

## 📋 CONFIGURATION

### Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OTLP collector endpoint | `http://localhost:4317` | No |
| `OTEL_SERVICE_NAME` | Service name for telemetry | `emqx-auth-service` | No |
| `OTEL_RESOURCE_ATTRIBUTES` | Additional resource attributes | - | No |
| `RUST_LOG` | Log level filter | `info` | No |

### OTLP Endpoint Formats

```bash
# gRPC (recommended)
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317

# With authentication
OTEL_EXPORTER_OTLP_ENDPOINT=http://user:pass@localhost:4317

# HTTPS for production
OTEL_EXPORTER_OTLP_ENDPOINT=https://otel-collector.example.com:4317
```

---

## 🔧 INTEGRATION WITH OBSERVABILITY BACKENDS

### 1. **Jaeger (Tracing)**

```yaml
# docker-compose.yml
version: '3'
services:
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "4317:4317"  # OTLP gRPC
      - "4318:4318"  # OTLP HTTP
      - "16686:16686" # UI
    environment:
      - COLLECTOR_OTLP_ENABLED=true
```

**Access UI:** http://localhost:16686

### 2. **Grafana Tempo (Tracing)**

```yaml
# tempo.yaml
server:
  http_listen_port: 3200

distributor:
  receivers:
    otlp:
      protocols:
        grpc:
          endpoint: "0.0.0.0:4317"

storage:
  trace:
    backend: local
    local:
      path: /tmp/tempo/blocks
```

```bash
docker run -p 4317:4317 -p 3200:3200 \
  -v $(pwd)/tempo.yaml:/etc/tempo.yaml \
  grafana/tempo:latest -config.file=/etc/tempo.yaml
```

### 3. **Prometheus (Metrics)**

```yaml
# prometheus.yaml
scrape_configs:
  - job_name: 'otel-collector'
    static_configs:
      - targets: ['otel-collector:8889']
```

### 4. **Grafana Loki (Logs)**

```yaml
# loki.yaml
auth_enabled: false

server:
  http_listen_port: 3100

common:
  path_prefix: /tmp/loki
  storage:
    filesystem:
      chunks_directory: /tmp/loki/chunks
      rules_directory: /tmp/loki/rules

limits_config:
  enforce_metric_name: false
  reject_old_samples: true
  reject_old_samples_max_age: 168h

schema_config:
  configs:
    - from: 2020-10-24
      store: boltdb-shipper
      object_store: filesystem
      schema: v11
      index:
        prefix: index_
        period: 24h
```

### 5. **Full Stack: Grafana Stack with OTLP**

```yaml
# docker-compose.yml
version: '3'
services:
  # OpenTelemetry Collector
  otel-collector:
    image: otel/opentelemetry-collector:latest
    command: ["--config=/etc/otel-collector-config.yaml"]
    volumes:
      - ./otel-collector-config.yaml:/etc/otel-collector-config.yaml
    ports:
      - "4317:4317"  # OTLP gRPC
      - "4318:4318"  # OTLP HTTP
    depends_on:
      - jaeger
      - loki
      - prometheus

  # Jaeger (Tracing)
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"
    environment:
      - COLLECTOR_OTLP_ENABLED=true

  # Loki (Logs)
  loki:
    image: grafana/loki:latest
    ports:
      - "3100:3100"
    command: -config.file=/etc/loki/local-config.yaml

  # Prometheus (Metrics)
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yaml:/etc/prometheus/prometheus.yml

  # Grafana (Dashboard)
  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_AUTH_ANONYMOUS_ENABLED=true
      - GF_AUTH_ANONYMOUS_ORG_ROLE=Admin
    volumes:
      - grafana-storage:/var/lib/grafana

volumes:
  grafana-storage:
```

---

## 📝 STRUCTURED LOGGING

### Log Format

All logs are now **JSON-formatted** with OpenTelemetry context:

```json
{
  "timestamp": "2026-03-14T10:30:00.123456Z",
  "level": "INFO",
  "target": "emqx_auth_service::services::create_mqtt_service",
  "message": "User MQTT created successfully",
  "span": {
    "username": "device_001"
  },
  "spans": [
    {
      "name": "POST /mqtt/create",
      "trace_id": "5bd66ef5095ebccd148dfa533aaeb980",
      "span_id": "6c292ca89b5bd37a"
    }
  ],
  "thread": "actix-rt|system:0|arbiter:1",
  "threadId": "1",
  "file": "src/services/create_mqtt_service.rs",
  "line": 42
}
```

### Log Levels

```rust
use tracing::{trace, debug, info, warn, error};

// Trace level - Very detailed debugging
trace!("Entering function with params: {:?}", params);

// Debug level - Debugging information
debug!("Processing request for user: {}", username);

// Info level - General operational information
info!("User created successfully"; "username" = %username);

// Warn level - Potential issues
warn!("Rate limit approaching for IP: {}", ip);

// Error level - Errors that need attention
error!("Failed to create user"; "error" = %e, "username" = %username);
```

### Structured Fields

```rust
// Add structured fields to logs
info!(
    "User authentication successful",
    username = %username,
    method = %auth_method,
    duration_ms = duration.as_millis()
);
```

---

## 🔍 DISTRIBUTED TRACING

### Manual Instrumentation

```rust
use tracing::{instrument, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

#[instrument(skip(self), fields(username = %dto.username))]
pub async fn create_mqtt(&self, dto: CreateMqttDTO) -> Result<bool, MqttServiceError> {
    // Add custom attributes to span
    Span::current().record("is_superuser", dto.is_superuser);
    
    // Add event to trace
    Span::current().add_event_with_timestamp(
        "User validation passed",
        Some(std::time::SystemTime::now()),
        vec![KeyValue::new("validation.duration_ms", 10)]
    );
    
    // ... business logic
}
```

### Trace Context Propagation

```rust
// Extract trace context from HTTP headers
use opentelemetry::global;
use opentelemetry::propagation::{Extractor, TextMapPropagator};

struct HeaderExtractor<'a>(&'a actix_web::HttpRequest);

impl<'a> Extractor for HeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.headers().get(key).and_then(|v| v.to_str().ok())
    }
    
    fn keys(&self) -> Vec<&str> {
        self.0.headers().keys().map(|k| k.as_str()).collect()
    }
}

let propagator = global::tracer_provider()
    .versioned_tracer("opentelemetry")
    .propagator();
let parent_cx = propagator.extract(&HeaderExtractor(&req));

// Set parent context
tracing::Span::current().set_parent(parent_cx);
```

---

## 📈 METRICS

### Available Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `http_requests_total` | Counter | Total HTTP requests |
| `http_request_duration_seconds` | Histogram | HTTP request duration |
| `auth_operations_total` | Counter | Authentication operations |
| `auth_operation_duration_seconds` | Histogram | Auth operation duration |
| `db_operations_total` | Counter | Database operations |
| `db_operation_duration_seconds` | Histogram | DB operation duration |

### Custom Metrics

```rust
use opentelemetry::global;
use opentelemetry::metrics::{Counter, Histogram};

// Create meter
let meter = global::meter("emqx-auth-service");

// Create counter
let request_counter: Counter<u64> = meter
    .u64_counter("auth_operations_total")
    .with_description("Total authentication operations")
    .init();

// Record metric
request_counter.add(1, &[KeyValue::new("operation", "create_user")]);

// Create histogram
let duration_histogram: Histogram<f64> = meter
    .f64_histogram("auth_operation_duration_seconds")
    .with_description("Authentication operation duration")
    .init();

// Record duration
duration_histogram.record(duration.as_secs_f64(), &[
    KeyValue::new("operation", "create_user")
]);
```

---

## 🎯 BEST PRACTICES

### 1. **Use Instrument Macro**

```rust
// ✅ Good: Automatic span creation
#[instrument(skip(self), fields(username = %dto.username))]
pub async fn create_user(&self, dto: CreateMqttDTO) -> Result<(), Error> {
    // ...
}

// ❌ Bad: Manual span management
pub async fn create_user(&self, dto: CreateMqttDTO) -> Result<(), Error> {
    let span = tracing::span!(tracing::Level::INFO, "create_user").entered();
    // ...
}
```

### 2. **Add Context to Spans**

```rust
// ✅ Good: Rich context
#[instrument(skip(self), fields(
    username = %dto.username,
    is_superuser = dto.is_superuser
))]
pub async fn create_user(&self, dto: CreateMqttDTO) -> Result<(), Error> {
    // ...
}

// ❌ Bad: No context
#[instrument(skip(self))]
pub async fn create_user(&self, dto: CreateMqttDTO) -> Result<(), Error> {
    // ...
}
```

### 3. **Log Errors with Context**

```rust
// ✅ Good: Error with context
match self.create_user(dto).await {
    Ok(_) => info!("User created"; "username" = %dto.username),
    Err(e) => error!(
        "Failed to create user";
        "username" = %dto.username,
        "error" = %e,
        "error_code" = ?e.code()
    ),
}

// ❌ Bad: Generic error
match self.create_user(dto).await {
    Ok(_) => info!("OK"),
    Err(e) => error!("Error: {}", e),
}
```

### 4. **Use Appropriate Log Levels**

```rust
// Trace: Very detailed, off by default
trace!("Function entry with params: {:?}", params);

// Debug: Debugging information
debug!("Cache miss for key: {}", key);

// Info: Normal operations
info!("Request processed successfully"; "duration_ms" = duration);

// Warn: Potential issues
warn!("Rate limit approaching"; "current" = count, "limit" = max);

// Error: Actual errors
error!("Database connection failed"; "error" = %e);
```

---

## 🔧 TROUBLESHOOTING

### Issue: No Traces in Jaeger

**Check:**
1. OTLP endpoint is correct
2. Port 4317 is accessible
3. Service name is set

```bash
# Test connectivity
curl -v http://localhost:4317

# Check logs
RUST_LOG=debug cargo run
```

### Issue: Logs Not Appearing in Loki

**Check:**
1. OTLP collector is running
2. Loki receiver is configured
3. Log level is appropriate

```yaml
# otel-collector-config.yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317

processors:
  batch:

exporters:
  loki:
    endpoint: http://loki:3100/loki/api/v1/push

service:
  pipelines:
    logs:
      receivers: [otlp]
      processors: [batch]
      exporters: [loki]
```

### Issue: High Memory Usage

**Solution:** Reduce batch size and increase export interval

```rust
let tracer_provider = sdktrace::TracerProvider::builder()
    .with_batch_exporter(
        trace_exporter,
        runtime::Tokio,
        sdktrace::BatchConfigBuilder::default()
            .with_max_queue_size(1024)
            .with_scheduled_delay(Duration::from_secs(5))
            .build()
    )
    .build();
```

---

## 📚 RESOURCES

- [OpenTelemetry Documentation](https://opentelemetry.io/docs/)
- [OpenTelemetry Rust SDK](https://github.com/open-telemetry/opentelemetry-rust)
- [OpenTelemetry Specification](https://github.com/open-telemetry/opentelemetry-specification)
- [Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/)

---

## 🎉 SUCCESS CRITERIA

You'll know OpenTelemetry is working when:

1. ✅ Logs appear in your logging backend (Loki, ELK, etc.)
2. ✅ Traces appear in your tracing backend (Jaeger, Tempo, etc.)
3. ✅ Metrics appear in your metrics backend (Prometheus, etc.)
4. ✅ Logs, traces, and metrics are correlated via trace IDs
5. ✅ Service name appears correctly in all backends

**Happy Observing!** 🔭
