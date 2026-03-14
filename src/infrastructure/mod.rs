//! Infrastructure layer - external concerns implementation

pub mod adapters;
pub mod metrics;
pub mod metrics_middleware;
pub mod persistence;
pub mod telemetry;

pub use adapters::{EncryptionAdapter, JwtAdapter};
pub use metrics::AppMetrics;
pub use metrics_middleware::MetricsMiddleware;
pub use persistence::{close_db, init_db, MqttUserRepositoryImpl};
