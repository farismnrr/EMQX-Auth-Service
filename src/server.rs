use actix_web::{App, HttpResponse, HttpServer, Responder, middleware, web};
use chrono::Local;
use log::{error, info};
use std::io::Write;
use std::sync::Arc;

use crate::infrastructure::mqtt_client::MqttClientManager;
use crate::infrastructure::mysql::{close_db, init_db, DbType};
use crate::middleware::api_key::ApiKeyMiddleware;
use crate::middleware::logger_request::RequestLoggerMiddleware;
use crate::middleware::powered_by::PoweredByMiddleware;

use crate::handler::create_mqtt_handler::{AppState as CreateMqttAppState, create_mqtt_handler};
use crate::handler::get_mqtt_credentials_handler::{
    AppState as GetCredentialsAppState, get_mqtt_credentials_handler,
};
use crate::handler::get_mqtt_list_handler::{
    AppState as GetListAppState, get_mqtt_by_id_handler, get_mqtt_list_handler,
};
use crate::handler::mqtt_acl_handler::{AppState as MqttAclAppState, mqtt_acl_handler};
use crate::handler::mqtt_login_handler::{
    AppState as MqttLoginAppState, login_with_credentials_handler,
};

use crate::services::create_mqtt_service::CreateMqttService;
use crate::services::get_mqtt_credentials_service::GetMqttCredentialsService;
use crate::services::get_mqtt_list_service::GetMqttListService;
use crate::services::mqtt_admin_service::MqttAdminService;
use crate::services::mqtt_acl_service::MqttAclService;
use crate::services::mqtt_login_service::MqttLoginService;

use crate::repositories::create_mqtt_repository::CreateMqttRepository;
use crate::repositories::get_mqtt_by_username_repository::GetMqttByUsernameRepository;
use crate::repositories::get_mqtt_list_repository::GetMqttListRepository;

async fn healthcheck() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body("OK")
}

pub async fn run_server() -> std::io::Result<()> {
    // =====================
    // 🌱 Load Environment Variables
    // =====================
    dotenvy::dotenv().ok();
    let env = env_logger::Env::new().filter_or("LOG_LEVEL", "info");
    let secret_key =
        std::env::var("SECRET_KEY").expect("❌ Environment variable SECRET_KEY is not set");

    let mysql_host = std::env::var("MYSQL_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let mysql_port = std::env::var("MYSQL_PORT")
        .unwrap_or_else(|_| "3306".to_string())
        .parse::<u16>()
        .unwrap_or(3306);
    let mysql_user = std::env::var("MYSQL_USER").expect("❌ MYSQL_USER is not set");
    let mysql_pass = std::env::var("MYSQL_PASSWORD").expect("❌ MYSQL_PASSWORD is not set");
    let mysql_db = std::env::var("MYSQL_DATABASE").expect("❌ MYSQL_DATABASE is not set");
    let db_type = DbType::from_str(&std::env::var("DB_TYPE").unwrap_or_else(|_| "mysql".to_string()));

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
    // 🐬 MySQL Initialization (Sea-ORM)
    // =====================
    let mysql_conn = init_db(db_type, &mysql_host, mysql_port, &mysql_user, &mysql_pass, &mysql_db)
        .await
        .map_err(|e| {
            error!("❌ Failed to initialize database via Sea-ORM: {}", e);
            std::io::Error::other("Failed to initialize database")
        })?;

    // =====================
    // 🧩 Repository Layer
    // =====================
    let create_mqtt_repo = Arc::new(CreateMqttRepository::new(mysql_conn.clone()));
    let get_mqtt_list_repo = Arc::new(GetMqttListRepository::new(mysql_conn.clone()));
    let get_by_username_repo = Arc::new(GetMqttByUsernameRepository::new(mysql_conn.clone()));

    // =====================
    // 🛠️ Service Layer
    // =====================
    let create_mqtt_service = Arc::new(CreateMqttService::new(
        Arc::clone(&create_mqtt_repo),
        Arc::clone(&get_by_username_repo),
    ));
    let get_mqtt_credentials_service = Arc::new(GetMqttCredentialsService::new(
        Arc::clone(&get_by_username_repo),
    ));
    let get_mqtt_list_service = Arc::new(GetMqttListService::new(Arc::clone(&get_mqtt_list_repo)));
    let mqtt_login_service = Arc::new(MqttLoginService::new(
        Arc::clone(&get_by_username_repo),
        secret_key,
    ));
    let mqtt_acl_service = Arc::new(MqttAclService::new(Arc::clone(&get_by_username_repo)));

    // =====================
    // 🤖 MQTT Admin Service
    // =====================
    let mqtt_admin_service = Arc::new(MqttAdminService::new(
        Arc::clone(&create_mqtt_repo),
        Arc::clone(&get_mqtt_list_repo),
    ));

    // =====================
    // 🌐 Load MQTT Configuration
    // =====================
    let mqtt_admin_enabled = std::env::var("MQTT_ADMIN_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase() == "true";
    let mqtt_broker_host = std::env::var("MQTT_BROKER_HOST")
        .unwrap_or_else(|_| "localhost".to_string());
    let mqtt_broker_port = std::env::var("MQTT_BROKER_PORT")
        .unwrap_or_else(|_| "1883".to_string())
        .parse::<u16>()
        .unwrap_or(1883);
    let mqtt_use_tls = std::env::var("MQTT_USE_TLS")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase() == "true";
    let mqtt_admin_username = std::env::var("MQTT_ADMIN_USERNAME").ok();
    let mqtt_admin_password = std::env::var("MQTT_ADMIN_PASSWORD").ok();
    let mqtt_use_shared_sub = std::env::var("MQTT_USE_SHARED_SUB")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase() == "true";

    // =====================
    // 🔌 Initialize MQTT Client (if enabled)
    // =====================
    let mut mqtt_client_manager: Option<MqttClientManager> = None;

    if mqtt_admin_enabled {
        info!("🔌 MQTT Admin API enabled, connecting to broker at {}:{} (TLS: {}, Shared Sub: {})", mqtt_broker_host, mqtt_broker_port, mqtt_use_tls, mqtt_use_shared_sub);

        match MqttClientManager::new(
            &mqtt_broker_host,
            mqtt_broker_port,
            "emqx_auth_admin",
            mqtt_admin_username.as_deref(),
            mqtt_admin_password.as_deref(),
            mqtt_use_tls,
            Arc::clone(&mqtt_admin_service),
        ).await {
            Ok(manager) => {
                info!("✅ MQTT Admin Client initialized successfully");
                mqtt_client_manager = Some(manager);
            }
            Err(e) => {
                error!("❌ Failed to initialize MQTT Admin Client: {}", e);
                error!("⚠️ MQTT Admin API will not be available");
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
    let get_mqtt_credentials_state = web::Data::new(GetCredentialsAppState {
        get_mqtt_credentials_service,
    });
    let mqtt_login_state = web::Data::new(MqttLoginAppState { mqtt_login_service });
    let mqtt_acl_state = web::Data::new(MqttAclAppState { mqtt_acl_service });
    let mysql_data = web::Data::new(mysql_conn.clone());

    // =====================
    // 🌐 Start Server
    // =====================
    info!("🚀 Actix server running on http://0.0.0.0:5500");
    let server = HttpServer::new(move || {
        App::new()
            .app_data(create_mqtt_state.clone())
            .app_data(get_mqtt_credentials_state.clone())
            .app_data(get_mqtt_list_state.clone())
            .app_data(mqtt_login_state.clone())
            .app_data(mqtt_acl_state.clone())
            .app_data(mysql_data.clone())
            .wrap(PoweredByMiddleware)
            .wrap(RequestLoggerMiddleware)
            .wrap(middleware::Compress::default())
            // 🩺 Root API — health check
            .route("/", web::get().to(healthcheck))
            // 👥 Mqtt endpoints
            .service(
                web::scope("/mqtt")
                    .wrap(ApiKeyMiddleware)
                    .route("/create", web::post().to(create_mqtt_handler))
                    .route("/check", web::post().to(login_with_credentials_handler))
                    .route("/credentials/{username}", web::get().to(get_mqtt_credentials_handler))
                    .route("/acl", web::post().to(mqtt_acl_handler))
                    .route("/{username}", web::delete().to(get_mqtt_by_id_handler))
                    // Development only
                    .route("", web::get().to(get_mqtt_list_handler))
                    .route("/{id}", web::get().to(get_mqtt_by_id_handler)),
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
    if let Some(ref mqtt_manager) = mqtt_client_manager {
        info!("🔌 Shutting down MQTT client...");
        mqtt_manager.shutdown().await;
    }

    info!("Closing database connection...");
    close_db(mysql_conn).await;

    server_result
}
