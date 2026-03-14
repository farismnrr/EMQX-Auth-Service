#![allow(dead_code)]

mod application;
mod config;
mod domain;
mod entities;
mod infrastructure;
mod presentation;
mod utils;

use actix_web::{web, HttpResponse, HttpServer, Responder};
use config::AppConfig;
use infrastructure::{close_db, init_db, EncryptionAdapter, JwtAdapter, MqttUserRepositoryImpl, AppMetrics, MetricsMiddleware};
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
        title = "EMQX Auth Service API",
        description = "Authentication and authorization service for EMQX MQTT broker",
        version = "0.1.0"
    ),
    tags(
        (name = "Health", description = "Health check endpoint"),
        (name = "MQTT", description = "MQTT authentication and authorization")
    ),
    paths(
        crate::presentation::handlers::rest::create_user_handler::create_user_handler,
        crate::presentation::handlers::rest::delete_user_handler::delete_user_handler,
        crate::presentation::handlers::rest::list_users_handler::list_users_handler,
        crate::presentation::handlers::rest::get_user_by_id_handler::get_user_by_id_handler,
        crate::presentation::handlers::rest::get_user_by_username_handler::get_user_by_username_handler,
        crate::presentation::handlers::emqx::auth_handler::emqx_auth_handler,
        crate::presentation::handlers::emqx::acl_handler::emqx_acl_handler,
    ),
    components(
        schemas(
            crate::presentation::handlers::rest::create_user_handler::CreateUserRequest,
            crate::presentation::handlers::emqx::auth_handler::EmqxAuthRequest,
            crate::presentation::handlers::emqx::auth_handler::EmqxAuthResponse,
            crate::presentation::handlers::emqx::acl_handler::EmqxAclRequest,
            crate::presentation::handlers::emqx::acl_handler::EmqxAclResponse,
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
    encryption: Arc<EncryptionAdapter>,
    jwt_adapter: Arc<JwtAdapter>,
    metrics: Arc<AppMetrics>,
}

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

    let encryption = EncryptionAdapter::new();
    let jwt_adapter = JwtAdapter::new(&config.secret_key, "emqx-auth-service");

    let server_state = Arc::new(ServerState {
        db: Arc::new(db_conn.clone()),
        repository: Arc::new(MqttUserRepositoryImpl::new(db_conn.clone())),
        encryption: Arc::new(encryption),
        jwt_adapter: Arc::new(jwt_adapter),
        metrics: Arc::new(metrics),
    });

    info!("🚀 Server running on http://0.0.0.0:5500");

    // Clone metrics for shutdown
    let metrics_for_shutdown = Arc::clone(&server_state.metrics);

    let server = HttpServer::new(move || {
        use actix_web::App;
        use actix_web::middleware;
        use crate::application::*;
        use crate::presentation::handlers::rest::*;
        use crate::presentation::handlers::rest::create_user_handler::CreateUserAppState;
        use crate::presentation::handlers::rest::delete_user_handler::DeleteUserAppState;
        use crate::presentation::handlers::rest::get_user_by_id_handler::GetByIdAppState;
        use crate::presentation::handlers::rest::get_user_by_username_handler::GetUserByUsernameAppState;
        use crate::presentation::handlers::rest::list_users_handler::ListUsersAppState;
        use crate::presentation::handlers::emqx::auth_handler::{EmqxAuthState, emqx_auth_handler};
        use crate::presentation::handlers::emqx::acl_handler::{EmqxAclState, emqx_acl_handler};

        App::new()
            .app_data(web::Data::new(Arc::clone(&server_state)))
            .wrap(MetricsMiddleware::new((*server_state.metrics).clone()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .app_data(web::Data::new(CreateUserAppState {
                use_case: CreateUserUseCase::new(
                    (*server_state.repository).clone(),
                    (*server_state.encryption).clone(),
                ),
            }))
            .app_data(web::Data::new(DeleteUserAppState {
                use_case: DeleteUserUseCase::new((*server_state.repository).clone()),
            }))
            .app_data(web::Data::new(GetByIdAppState {
                use_case: GetUserUseCase::new((*server_state.repository).clone()),
            }))
            .app_data(web::Data::new(GetUserByUsernameAppState {
                use_case: GetUserUseCase::new((*server_state.repository).clone()),
            }))
            .app_data(web::Data::new(ListUsersAppState {
                use_case: ListUsersUseCase::new((*server_state.repository).clone()),
            }))
            .app_data(web::Data::new(EmqxAuthState {
                use_case: AuthenticateUserUseCase::new(
                    (*server_state.repository).clone(),
                    (*server_state.jwt_adapter).clone(),
                ),
                metrics: (*server_state.metrics).clone(),
            }))
            .app_data(web::Data::new(EmqxAclState {
                use_case: CheckAclUseCase::new((*server_state.repository).clone()),
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
                    .route("/create", web::post().to(create_user_handler))
                    .route("/users/{username}", web::get().to(get_user_by_username_handler))
                    .route("/{username}", web::delete().to(delete_user_handler))
                    .route("", web::get().to(list_users_handler))
                    .route("/{id}", web::get().to(get_user_by_id_handler)),
            )
            // EMQX native endpoints - format compatible with EMQX HTTP plugin
            .service(
                web::scope("/emqx")
                    .route("/auth", web::post().to(emqx_auth_handler))
                    .route("/acl", web::post().to(emqx_acl_handler)),
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
