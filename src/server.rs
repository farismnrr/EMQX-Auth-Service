use actix_web::{middleware, web, App, HttpResponse, HttpServer, Responder};
use chrono::Local;
use log::{error, info, warn};
use std::io::Write;
use std::sync::Arc;

use crate::handler::mqtt::MqttHandler;
use crate::infrastructure::database_infrastructure::{close_db, init_db};
use crate::middleware::api_key_middleware::ApiKeyMiddleware;
use crate::middleware::logger_request_middleware::RequestLoggerMiddleware;
use crate::middleware::powered_by_middleware::PoweredByMiddleware;
use crate::middleware::rate_limit_middleware::RateLimiter;

use crate::handler::rest::check_acl_handler::{check_acl_handler, AppState as MqttAclAppState};
use crate::handler::rest::check_login_handler::{check_login_handler, AppState as MqttLoginAppState};
use crate::handler::rest::create_user_handler::{create_user_handler, AppState as CreateMqttAppState};
use crate::handler::rest::delete_user_handler::delete_user_handler;
use crate::handler::rest::get_user_by_id_handler::get_user_by_id_handler;
use crate::handler::rest::get_user_by_username_handler::{
    get_user_by_username_handler, AppState as GetUserByUsernameAppState,
};
use crate::handler::rest::list_users_handler::{list_users_handler, AppState as GetListAppState};

use crate::services::create_mqtt_service::CreateMqttService;
use crate::services::get_mqtt_credentials_service::GetMqttCredentialsService;
use crate::services::get_mqtt_list_service::GetMqttListService;
use crate::services::mqtt_acl_service::MqttAclService;
use crate::services::mqtt_admin_service::MqttAdminService;
use crate::services::mqtt_login_service::MqttLoginService;

use crate::repositories::create_mqtt_repository::CreateMqttRepository;
use crate::repositories::get_mqtt_by_username_repository::GetMqttByUsernameRepository;
use crate::repositories::get_mqtt_list_repository::GetMqttListRepository;

async fn healthcheck(db_conn: web::Data<sea_orm::DatabaseConnection>) -> impl Responder {
    match db_conn.ping().await {
        Ok(_) => HttpResponse::Ok()
            .content_type("text/plain; charset=utf-8")
            .body("OK"),
        Err(e) => {
            error!("Healthcheck failed: Database connection error: {}", e);
            HttpResponse::InternalServerError()
                .content_type("text/plain; charset=utf-8")
                .body("ERROR: Database connection failed")
        }
    }
}

pub async fn run_server() -> std::io::Result<()> {
    // =====================
    // 🌱 Load Environment Variables
    // =====================
    dotenvy::dotenv().ok();
    let env = env_logger::Env::new().filter_or("LOG_LEVEL", "info");
    let secret_key =
        std::env::var("SECRET_KEY").expect("❌ Environment variable SECRET_KEY is not set");

    let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "mqtt_auth.sqlite".to_string());

    // =====================
    // 🪵 Initialize logger with custom format + color
    // =====================
    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            let ts = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let color = match record.level() {
                log::Level::Error => "\x1b[31m", // Red
                log::Level::Warn => "\x1b[33m",  // Yellow
                log::Level::Info => "\x1b[32m",  // Green
                log::Level::Debug => "\x1b[34m", // Blue
                log::Level::Trace => "\x1b[36m", // Cyan
            };
            let reset = "\x1b[0m";

            writeln!(
                buf,
                "[{} {}{:<5}{}] {}",
                ts,
                color,
                record.level(),
                reset,
                record.args()
            )
        })
        .format_target(false)
        .init();
    info!("🟢 Logging initialized successfully");

    // =====================
    // 📁 SQLite Initialization (Sea-ORM)
    // =====================
    let db_conn = init_db(&db_path)
    .await
    .map_err(|e| {
        error!("❌ Failed to initialize database via Sea-ORM: {}", e);
        std::io::Error::other("Failed to initialize database")
    })?;

    // =====================
    // 🧩 Repository Layer
    // =====================
    let create_mqtt_repo = Arc::new(CreateMqttRepository::new(db_conn.clone()));
    let get_mqtt_list_repo = Arc::new(GetMqttListRepository::new(db_conn.clone()));
    let get_by_username_repo = Arc::new(GetMqttByUsernameRepository::new(db_conn.clone()));

    // =====================
    // 🛠️ Service Layer
    // =====================
    let create_mqtt_service = Arc::new(CreateMqttService::new(
        Arc::clone(&create_mqtt_repo),
        Arc::clone(&get_by_username_repo),
    ));
    let get_mqtt_credentials_service = Arc::new(GetMqttCredentialsService::new(Arc::clone(
        &get_by_username_repo,
    )));
    let get_mqtt_list_service = Arc::new(GetMqttListService::new(Arc::clone(&get_mqtt_list_repo)));
    let mqtt_login_service = Arc::new(MqttLoginService::new(
        Arc::clone(&get_mqtt_credentials_service),
        secret_key.clone(),
    ));
    let mqtt_acl_service = Arc::new(MqttAclService::new(Arc::clone(&get_by_username_repo)));

    // =====================
    // 🤖 MQTT Admin Service
    // =====================
    let mqtt_admin_service = Arc::new(MqttAdminService::new(
        Arc::clone(&create_mqtt_repo),
        Arc::clone(&get_mqtt_list_repo),
        Arc::clone(&get_mqtt_credentials_service),
        secret_key,
    ));

    // =====================
    // 🌐 Load MQTT Configuration
    // =====================
    let mqtt_admin_enabled = std::env::var("MQTT_ADMIN_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase()
        == "true";
    let mqtt_broker_host =
        std::env::var("MQTT_BROKER_HOST").unwrap_or_else(|_| "localhost".to_string());
    let mqtt_broker_port = std::env::var("MQTT_BROKER_PORT")
        .unwrap_or_else(|_| "1883".to_string())
        .parse::<u16>()
        .unwrap_or(1883);
    let mqtt_use_tls = std::env::var("MQTT_USE_TLS")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase()
        == "true";
    let mqtt_admin_username = std::env::var("MQTT_ADMIN_USERNAME").ok();
    let mqtt_admin_password = std::env::var("MQTT_ADMIN_PASSWORD").ok();
    let mqtt_use_shared_sub = std::env::var("MQTT_USE_SHARED_SUB")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase()
        == "true";

    // =====================
    // 🔌 Initialize MQTT Client (if enabled)
    // =====================
    let mut mqtt_handler: Option<MqttHandler> = None;

    if mqtt_admin_enabled {
        // Generate unique client ID to avoid collisions in multi-instance deployments
        let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string());
        let random_suffix = uuid::Uuid::new_v4()
            .to_string()
            .split('-')
            .next()
            .unwrap_or("rand")
            .to_string();
        let default_client_id = format!("emqx_auth_admin-{}-{}", hostname, random_suffix);
        let mqtt_client_id = std::env::var("MQTT_ADMIN_CLIENT_ID").unwrap_or(default_client_id);

        info!(
            "🔌 MQTT Admin API enabled, connecting to broker at {}:{} (TLS: {}, Shared Sub: {}, ClientID: {})",
            mqtt_broker_host, mqtt_broker_port, mqtt_use_tls, mqtt_use_shared_sub, mqtt_client_id
        );

        match MqttHandler::new(
            &mqtt_broker_host,
            mqtt_broker_port,
            &mqtt_client_id,
            mqtt_admin_username.as_deref(),
            mqtt_admin_password.as_deref(),
            mqtt_use_tls,
            mqtt_use_shared_sub,
            Arc::clone(&mqtt_admin_service),
        )
        .await
        {
            Ok(handler) => {
                info!("✅ MQTT Admin Client initialized successfully");
                mqtt_handler = Some(handler);
            }
            Err(e) => {
                warn!("⚠️ Failed to initialize MQTT Admin Client: {}", e);
                warn!("⚠️ MQTT Admin API will not be available but service will continue to run");
            }
        }
    } else {
        info!("ℹ️ MQTT Admin API is disabled (set MQTT_ADMIN_ENABLED=true to enable)");
    }

    // =====================
    // 🚀 App State
    // =====================
    let create_mqtt_state = web::Data::new(CreateMqttAppState {
        create_mqtt_service,
    });
    let get_mqtt_list_state = web::Data::new(GetListAppState {
        get_mqtt_list_service,
    });
    let get_user_by_username_state = web::Data::new(GetUserByUsernameAppState {
        get_mqtt_credentials_service: Arc::clone(&get_mqtt_credentials_service),
    });
    let mqtt_login_state = web::Data::new(MqttLoginAppState { mqtt_login_service });
    let mqtt_acl_state = web::Data::new(MqttAclAppState { mqtt_acl_service });
    let db_data = web::Data::new(db_conn.clone());
    let mqtt_admin_data = web::Data::new(Arc::clone(&mqtt_admin_service));

    // =====================
    // 📖 OpenAPI Documentation
    // =====================
    use utoipa::OpenApi;
    use utoipa_scalar::{Scalar, Servable};

    #[derive(OpenApi)]
    #[openapi(
        paths(
            crate::handler::rest::create_user_handler::create_user_handler,
            crate::handler::rest::check_login_handler::check_login_handler,
            crate::handler::rest::check_acl_handler::check_acl_handler,
            crate::handler::rest::list_users_handler::list_users_handler,
            crate::handler::rest::get_user_by_id_handler::get_user_by_id_handler,
            crate::handler::rest::get_user_by_username_handler::get_user_by_username_handler,
            crate::handler::rest::delete_user_handler::delete_user_handler,
        ),
        components(
            schemas(
                crate::dtos::mqtt_dto::MqttDTO,
                crate::dtos::mqtt_dto::CreateMqttDTO,
                crate::dtos::mqtt_dto::MqttLoginDTO,
                crate::dtos::mqtt_dto::AuthType,
                crate::dtos::mqtt_dto::MqttAclDTO,
                crate::dtos::mqtt_dto::MqttCredentialsDTO,
                crate::dtos::mqtt_dto::GetMqttListDTO,
                crate::dtos::mqtt_dto::PaginationInfo,
                crate::dtos::mqtt_dto::GetMqttListPaginatedDTO,
                crate::dtos::mqtt_admin_dto::AdminUserResponse,
                crate::dtos::response_dto::ResponseDTO<'static>,
                crate::dtos::response_dto::ErrorResponseDTO<'static>,
                crate::dtos::response_dto::ErrorResponseValidation,
                crate::services::service_error::ValidationError,
            )
        ),
        tags(
            (name = "MQTT", description = "MQTT Management Endpoints")
        ),
        modifiers(&SecurityAddon)
    )]
    struct ApiDoc;

    struct SecurityAddon;

    impl utoipa::Modify for SecurityAddon {
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

    // =====================
    // 🌐 Start Server
    // =====================
    let rate_limit_rpm = std::env::var("MQTT_AUTH_RATE_LIMIT")
        .unwrap_or_else(|_| "100".to_string())
        .parse::<u32>()
        .unwrap_or(100);

    info!("🚀 Actix server running on http://0.0.0.0:5500");
    let server = HttpServer::new(move || {
        App::new()
            .app_data(create_mqtt_state.clone())
            .app_data(get_mqtt_list_state.clone())
            .app_data(get_user_by_username_state.clone())
            .app_data(mqtt_login_state.clone())
            .app_data(mqtt_acl_state.clone())
            .app_data(db_data.clone())
            .app_data(mqtt_admin_data.clone())
            .wrap(PoweredByMiddleware)
            .wrap(RequestLoggerMiddleware)
            .wrap(middleware::Compress::default())
            // 📖 Scalar UI
            .service(Scalar::with_url("/openapi", ApiDoc::openapi()))
            // 🩺 Root API — health check
            .route("/", web::get().to(healthcheck))
            // 👥 Mqtt endpoints
            .service(
                web::scope("/mqtt")
                    .wrap(ApiKeyMiddleware)
                    .wrap(RateLimiter::new(rate_limit_rpm))
                    .route("/create", web::post().to(create_user_handler))
                    .route("/check", web::post().to(check_login_handler))
                    .route(
                        "/users/{username}",
                        web::get().to(get_user_by_username_handler),
                    )
                    .route("/acl", web::post().to(check_acl_handler))
                    .route("/{username}", web::delete().to(delete_user_handler))
                    // Development only
                    .route("", web::get().to(list_users_handler))
                    .route("/{id}", web::get().to(get_user_by_id_handler)),
            )
    })
    .bind(("0.0.0.0", 5500))?
    .run();

    let server_handle = server.handle();

    // Handle graceful shutdown signals
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for ctrl-c");
        info!("Signals received, starting graceful shutdown...");
        server_handle.stop(true).await;
    });

    let server_result = server.await.map_err(|e| {
        error!("❌ Server error: {}", e);
        e
    });

    // =====================
    // 🧹 Cleanup
    // =====================
    info!("Shutting down server...");

    // Shutdown MQTT client if enabled
    if let Some(ref handler) = mqtt_handler {
        info!("🔌 Shutting down MQTT client...");
        handler.shutdown().await;
    }

    info!("Closing database connection...");
    close_db(db_conn).await;

    server_result
}
