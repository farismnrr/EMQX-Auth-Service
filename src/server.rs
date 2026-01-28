use actix_web::{App, HttpResponse, HttpServer, Responder, middleware, web};
use chrono::Local;
use log::{error, info};
use std::io::Write;
use std::sync::Arc;

use crate::infrastructure::postgres;
use crate::infrastructure::rocksdb::{close_rocksdb, init_rocksdb};
use crate::middleware::api_key::ApiKeyMiddleware;
use crate::middleware::logger_request::RequestLoggerMiddleware;
use crate::middleware::powered_by::PoweredByMiddleware;

use crate::handler::create_mqtt_handler::create_mqtt_handler;
use crate::handler::get_mqtt_list_handler::{AppState as GetListAppState, get_mqtt_list_handler};
use crate::handler::mqtt_acl_handler::mqtt_acl_handler;
use crate::handler::mqtt_login_handler::login_with_credentials_handler;
use crate::handler::soft_delete_mqtt_handler::soft_delete_mqtt;

use crate::repositories::cache_repository::CacheRepository;
use crate::repositories::create_mqtt_repository::CreateMqttRepository;
use crate::repositories::get_mqtt_by_username_repository::GetMqttByUsernameRepository;
use crate::repositories::get_mqtt_list_repository::GetMqttListRepository;
use crate::repositories::soft_delete_mqtt_repository::SoftDeleteMqttRepository;

use crate::services::create_mqtt_service::CreateMqttService;
use crate::services::get_mqtt_list_service::GetMqttListService;
use crate::services::mqtt_acl_service::MqttAclService;
use crate::services::mqtt_login_service::MqttLoginService;
use crate::services::soft_delete_mqtt_service::SoftDeleteMqttService;

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
    let db_path = std::env::var("DB_PATH").expect("❌ Environment variable DB_PATH is not set");
    let secret_key =
        std::env::var("SECRET_KEY").expect("❌ Environment variable SECRET_KEY is not set");

    // =====================
    // 🪵 Initialize logger
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
    // 🗄️ Database Initialization
    // =====================
    let pg_pool = Arc::new(postgres::connect().await);
    info!("🟢 Postgres connection pool established");

    info!("🔄 Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&*pg_pool)
        .await
        .map_err(|e| {
            error!("❌ Migration failed: {}", e);
            std::io::Error::other("Migration failed")
        })?;
    info!("✅ Database migrations applied successfully");

    let rocksdb = init_rocksdb(&db_path).map_err(|e| {
        error!("❌ Failed to initialize RocksDB at {}: {}", db_path, e);
        std::io::Error::other("Failed to initialize RocksDB")
    })?;
    info!("🟢 RocksDB (Cache) initialized successfully at {}", db_path);

    // =====================
    // 🧩 Repository Layer
    // =====================
    let cache_repo = Arc::new(CacheRepository::new(Arc::clone(&rocksdb)));

    let create_mqtt_repo = Arc::new(CreateMqttRepository::new(
        Arc::clone(&pg_pool),
        Arc::clone(&cache_repo),
    ));
    let get_mqtt_list_repo = Arc::new(GetMqttListRepository::new(Arc::clone(&pg_pool)));
    let get_by_username_repo = Arc::new(GetMqttByUsernameRepository::new(
        Arc::clone(&pg_pool),
        Arc::clone(&cache_repo),
    ));
    let soft_delete_mqtt_repo = Arc::new(SoftDeleteMqttRepository::new(
        Arc::clone(&pg_pool),
        Arc::clone(&cache_repo),
    ));

    // =====================
    // 🛠️ Service Layer
    // =====================
    let create_mqtt_service = Arc::new(CreateMqttService::new(
        Arc::clone(&create_mqtt_repo),
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
    let create_mqtt_data = web::Data::new(create_mqtt_service);
    let get_mqtt_list_data = web::Data::new(GetListAppState {
        get_mqtt_list_service,
    });
    let mqtt_login_data = web::Data::new(mqtt_login_service);
    let mqtt_acl_data = web::Data::new(mqtt_acl_service);
    let soft_delete_mqtt_data = web::Data::new(soft_delete_mqtt_service);

    // =====================
    // 🌐 Start Server
    // =====================
    info!("🚀 Actix server running on http://0.0.0.0:5500");
    let server_result = HttpServer::new(move || {
        App::new()
            .app_data(create_mqtt_data.clone())
            .app_data(mqtt_login_data.clone())
            .app_data(mqtt_acl_data.clone())
            .app_data(soft_delete_mqtt_data.clone())
            .app_data(get_mqtt_list_data.clone())
            .wrap(PoweredByMiddleware)
            .wrap(RequestLoggerMiddleware)
            .wrap(middleware::Compress::default())
            .route("/", web::get().to(healthcheck))
            .service(
                web::scope("/mqtt")
                    .wrap(ApiKeyMiddleware)
                    .route("/create", web::post().to(create_mqtt_handler))
                    .route("/check", web::post().to(login_with_credentials_handler))
                    .route("/acl", web::post().to(mqtt_acl_handler))
                    .route("/{username}", web::delete().to(soft_delete_mqtt))
                    .route("", web::get().to(get_mqtt_list_handler)),
            )
    })
    .bind(("0.0.0.0", 5500))?
    .run()
    .await;

    // cleanup
    close_rocksdb(rocksdb);
    server_result
}
