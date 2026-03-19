#![allow(dead_code)]

mod application;
mod config;
mod domain;
mod infrastructure;
mod presentation;
mod utils;

use actix_web::{web, HttpResponse, HttpServer, Responder};
use config::AppConfig;
use infrastructure::{close_db, init_db, MqttUserRepositoryImpl, AppMetrics};
use presentation::{MetricsMiddleware, ApiKeyMiddleware};
use infrastructure::telemetry::{init_opentelemetry_traces, shutdown_opentelemetry_traces};
use std::sync::Arc;
use tracing::{error, info};
use utoipa::{Modify, OpenApi};
use utoipa_scalar::{Scalar, Servable};

/// Security addon - adds x-api-key authentication to OpenAPI
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                utoipa::openapi::security::SecurityScheme::ApiKey(
                    utoipa::openapi::security::ApiKey::Header(
                        utoipa::openapi::security::ApiKeyValue::new("x-api-key"),
                    ),
                ),
            );
        }
    }
}

/// OpenAPI specification
#[derive(OpenApi)]
#[openapi(
    info(
        title = "MQTT User Management API",
        description = "Simple REST API for MQTT user management",
        version = "0.1.0"
    ),
    tags(
        (name = "Health", description = "Health check endpoint"),
        (name = "Users", description = "User management operations")
    ),
    paths(
        healthcheck,
        crate::presentation::handlers::rest::create_user_handler::create_user_handler,
        crate::presentation::handlers::rest::delete_user_handler::delete_user_handler,
        crate::presentation::handlers::rest::list_users_handler::list_users_handler,
        crate::presentation::handlers::rest::jwt_handler::jwt_handler,
    ),
    components(
        schemas(
            crate::presentation::handlers::rest::create_user_handler::CreateUserRequest,
            crate::presentation::handlers::rest::jwt_handler::JwtRequest,
            crate::presentation::handlers::rest::jwt_handler::JwtResponseData,
            crate::application::SuccessResponseJson,
            crate::application::ErrorResponse,
            crate::application::UserDTO,
            crate::application::UserListDTO,
        )
    ),
    modifiers(&SecurityAddon),
    security(
        ("api_key" = [])
    )
)]
struct ApiDoc;

/// Shared application state
struct ServerState {
    db: Arc<sea_orm::DatabaseConnection>,
    repository: Arc<MqttUserRepositoryImpl>,
    metrics: Arc<AppMetrics>,
}

#[utoipa::path(
    get,
    path = "/",
    tag = "Health",
    operation_id = "healthcheck",
    summary = "Health check endpoint",
    description = "Returns OK if the service is healthy, ERROR otherwise",
    responses(
        (status = 200, description = "Service is healthy", body = str,
            example = json!("OK")),
        (status = 500, description = "Service is unhealthy", body = str,
            example = json!("ERROR")),
    ),
)]
async fn healthcheck(db: web::Data<Arc<sea_orm::DatabaseConnection>>) -> impl Responder {
    match db.ping().await {
        Ok(_) => HttpResponse::Ok().content_type("text/plain").body("OK"),
        Err(_) => HttpResponse::InternalServerError()
            .content_type("text/plain")
            .body("ERROR"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = match AppConfig::from_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("❌ FATAL: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize OpenTelemetry traces
    if let Err(e) = init_opentelemetry_traces(&config.otlp_service_name, &config.otlp_endpoint) {
        eprintln!("❌ FATAL: Failed to initialize OpenTelemetry Traces: {}", e);
        std::process::exit(1);
    }

    // Initialize OpenTelemetry metrics
    let metrics = match AppMetrics::new(&config.otlp_service_name, &config.otlp_endpoint) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("❌ FATAL: Failed to initialize OpenTelemetry Metrics: {}", e);
            std::process::exit(1);
        }
    };

    info!("📝 Logging level: {}", config.log_level);

    let db_conn = init_db(&config.database).await.map_err(|e| {
        error!("❌ Failed to initialize database: {}", e);
        std::io::Error::other("Database initialization failed")
    })?;

    let server_state = Arc::new(ServerState {
        db: Arc::new(db_conn.clone()),
        repository: Arc::new(MqttUserRepositoryImpl::new(db_conn.clone())),
        metrics: Arc::new(metrics),
    });

    info!("🚀 Server running on http://0.0.0.0:5500");

    // Clone metrics for shutdown
    let metrics_for_shutdown = Arc::clone(&server_state.metrics);

    let api_key = config.api_key.clone();
    let secret_key = config.secret_key.clone();
    let server = HttpServer::new(move || {
        use actix_web::App;
        use actix_web::middleware;
        use crate::application::*;
        use crate::application::use_cases::GenerateJwtUseCase;
        use crate::presentation::handlers::rest::*;
        use crate::presentation::handlers::rest::create_user_handler::CreateUserAppState;
        use crate::presentation::handlers::rest::delete_user_handler::DeleteUserAppState;
        use crate::presentation::handlers::rest::list_users_handler::ListUsersAppState;
        use crate::presentation::handlers::rest::jwt_handler::JwtAppState;

        App::new()
            .app_data(web::Data::new(Arc::clone(&server_state)))
            .wrap(MetricsMiddleware::new((*server_state.metrics).clone()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .app_data(web::Data::new(CreateUserAppState {
                use_case: CreateUserUseCase::new((*server_state.repository).clone()),
            }))
            .app_data(web::Data::new(DeleteUserAppState {
                use_case: DeleteUserUseCase::new((*server_state.repository).clone()),
            }))
            .app_data(web::Data::new(ListUsersAppState {
                use_case: ListUsersUseCase::new((*server_state.repository).clone()),
            }))
            .app_data(web::Data::new(JwtAppState {
                use_case: GenerateJwtUseCase::new((*server_state.repository).clone(), secret_key.clone()),
            }))
            .app_data(web::Data::new(Arc::clone(&server_state.db)))
            // 📖 Scalar UI
            .service(Scalar::with_url("/openapi", ApiDoc::openapi()))
            // 🩺 Health check
            .route("/", web::get().to(healthcheck))
            .route("/api-docs/openapi.json", web::get().to(|| async {
                actix_web::web::Json(ApiDoc::openapi())
            }))
            .service(
                web::scope("/mqtt")
                    .wrap(ApiKeyMiddleware::new(api_key.clone()))
                    .route("/create", web::post().to(create_user_handler))
                    .route("", web::get().to(list_users_handler))
                    .route("/{username}", web::delete().to(delete_user_handler))
                    .route("/jwt", web::post().to(jwt_handler)),
            )
    })
    .bind(("0.0.0.0", 5500))?
    .run();

    let server_handle = server.handle();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for ctrl-c");
        info!("Shutting down...");
        server_handle.stop(true).await;
    });

    let server_result = server.await;

    info!("Closing database...");
    close_db(db_conn).await;

    info!("Shutting down OpenTelemetry...");
    shutdown_opentelemetry_traces();
    metrics_for_shutdown.shutdown();

    server_result
}
