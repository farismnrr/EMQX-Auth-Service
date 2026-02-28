use actix_web::{App, HttpResponse, HttpServer, Responder, middleware, web};
use chrono::Local;
use log::{error, info};
use std::io::Write;
use std::sync::Arc;

use crate::infrastructure::database::{DbType, close_db, init_db};
use crate::middleware::api_key::ApiKeyMiddleware;
use crate::middleware::logger_request::RequestLoggerMiddleware;
use crate::middleware::powered_by::PoweredByMiddleware;

use crate::handler::create_mqtt_handler::{AppState as CreateMqttAppState, create_mqtt_handler};
use crate::handler::get_mqtt_credentials_handler::{
    AppState as GetCredentialsAppState, get_mqtt_credentials_handler,
};
use crate::handler::get_mqtt_list_handler::{AppState as GetListAppState, get_mqtt_list_handler};
use crate::handler::mqtt_acl_handler::{AppState as MqttAclAppState, mqtt_acl_handler};
use crate::handler::mqtt_login_handler::{
    AppState as MqttLoginAppState, login_with_credentials_handler,
};
use crate::handler::soft_delete_mqtt_handler::{
    AppState as SoftDeleteMqttAppState, soft_delete_mqtt,
};

use crate::services::create_mqtt_service::CreateMqttService;
use crate::services::get_mqtt_credentials_service::GetMqttCredentialsService;
use crate::services::get_mqtt_list_service::GetMqttListService;
use crate::services::mqtt_acl_service::MqttAclService;
use crate::services::mqtt_login_service::MqttLoginService;
use crate::services::soft_delete_mqtt_service::SoftDeleteMqttService;

use crate::repositories::create_mqtt_repository::CreateMqttRepository;
use crate::repositories::get_mqtt_by_username_repository::GetMqttByUsernameRepository;
use crate::repositories::get_mqtt_list_repository::GetMqttListRepository;
use crate::repositories::soft_delete_mqtt_repository::SoftDeleteMqttRepository;

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

    let db_type_str = std::env::var("DB_TYPE").unwrap_or_else(|_| "mysql".to_string());
    let db_type = DbType::from_str(&db_type_str);

    let db_host = std::env::var(if matches!(db_type, DbType::Postgres) { "POSTGRES_HOST" } else { "MYSQL_HOST" })
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    let db_port = std::env::var(if matches!(db_type, DbType::Postgres) { "POSTGRES_PORT" } else { "MYSQL_PORT" })
        .unwrap_or_else(|_| if matches!(db_type, DbType::Postgres) { "5432" } else { "3306" }.to_string())
        .parse::<u16>()
        .unwrap_or(if matches!(db_type, DbType::Postgres) { 5432 } else { 3306 });
    let db_user = std::env::var(if matches!(db_type, DbType::Postgres) { "POSTGRES_USER" } else { "MYSQL_USER" })
        .expect("❌ Database user is not set");
    let db_pass = std::env::var(if matches!(db_type, DbType::Postgres) { "POSTGRES_PASSWORD" } else { "MYSQL_PASSWORD" })
        .expect("❌ Database password is not set");
    let db_name = std::env::var(if matches!(db_type, DbType::Postgres) { "POSTGRES_DATABASE" } else { "MYSQL_DATABASE" })
        .expect("❌ Database name is not set");

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
    // 🗄️ Database Initialization (Sea-ORM)
    // =====================
    let db_conn = init_db(db_type, &db_host, db_port, &db_user, &db_pass, &db_name)
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
    let soft_delete_mqtt_repo = Arc::new(SoftDeleteMqttRepository::new(db_conn.clone()));

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
    let soft_delete_mqtt_service = Arc::new(SoftDeleteMqttService::new(
        Arc::clone(&get_by_username_repo),
        Arc::clone(&soft_delete_mqtt_repo),
    ));

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
    let soft_delete_mqtt_state = web::Data::new(SoftDeleteMqttAppState {
        soft_delete_mqtt_service,
    });
    let mysql_data = web::Data::new(db_conn.clone());

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
            .app_data(soft_delete_mqtt_state.clone())
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
                    .route("/{username}", web::delete().to(soft_delete_mqtt))
                    // Development only
                    .route("", web::get().to(get_mqtt_list_handler)),
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

    info!("Closing database connection...");
    close_db(db_conn).await;

    server_result
}
