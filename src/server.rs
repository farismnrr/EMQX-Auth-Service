use actix_web::{App, HttpServer, HttpResponse, middleware, web, Responder};
use chrono::Local;
use std::sync::Arc;
use std::io::Write;
use log::{info, error};

use crate::infrastructure::rocksdb::{init_rocksdb, close_rocksdb};
use crate::middleware::api_key::ApiKeyMiddleware;
use crate::middleware::powered_by::PoweredByMiddleware;
use crate::middleware::logger_request::RequestLoggerMiddleware;

use crate::handler::create_user_handler::{create_user_handler, AppState as CreateUserAppState};
use crate::handler::get_user_list_handler::{get_user_list_handler, AppState as GetListAppState};
use crate::handler::check_user_active_handler::{check_user_active_handler, AppState as CheckUserActiveAppState};

use crate::services::create_user_service::CreateUserService;
use crate::services::get_user_list_service::GetUserListService;
use crate::services::check_user_active_service::CheckUserActiveService;

use crate::repositories::create_user_repository::CreateUserRepository;
use crate::repositories::get_user_list_repository::GetUserListRepository;
use crate::repositories::check_user_active_repository::CheckUserActiveRepository;

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

    // =====================
    // 🪵 Initialize logger with custom format + color
    // =====================
    let env = env_logger::Env::new().filter_or("LOG_LEVEL", "info");
    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            let ts = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let color = match record.level() {
                log::Level::Error => "\x1b[31m", // Red
                log::Level::Warn  => "\x1b[33m", // Yellow
                log::Level::Info  => "\x1b[32m", // Green
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
    let db_path = std::env::var("DB_PATH")
        .expect("❌ Environment variable DB_PATH is not set");
    let db = init_rocksdb(&db_path)
        .map_err(|e| {
            error!("❌ Failed to initialize RocksDB at {}: {}", db_path, e);
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to initialize RocksDB")
        })?;
    info!("🟢 RocksDB initialized successfully at {}", db_path);

    // =====================
    // 🧩 Repository Layer
    // =====================
    let create_repo = Arc::new(CreateUserRepository::new(Arc::clone(&db)));
    let list_repo = Arc::new(GetUserListRepository::new(Arc::clone(&db)));
    let check_user_repo = Arc::new(CheckUserActiveRepository::new(Arc::clone(&db)));

    // =====================
    // 🛠️ Service Layer
    // =====================
    let create_user_service = Arc::new(CreateUserService::new(Arc::clone(&create_repo)));
    let get_user_list_service = Arc::new(GetUserListService::new(Arc::clone(&list_repo)));
    let check_user_active_service = Arc::new(CheckUserActiveService::new(Arc::clone(&check_user_repo)));

    // =====================
    // 🚀 App State
    // =====================
    let create_state = web::Data::new(CreateUserAppState { create_user_service });
    let list_state = web::Data::new(GetListAppState { get_user_list_service });
    let check_state = web::Data::new(CheckUserActiveAppState { check_user_active_service });

    // =====================
    // 🌐 Start Server
    // =====================
    info!("🚀 Actix server running on http://0.0.0.0:5500");
    let server_result = HttpServer::new(move || {
        App::new()
            .app_data(create_state.clone())
            .app_data(list_state.clone())
            .app_data(check_state.clone())
            .wrap(ApiKeyMiddleware)
            .wrap(PoweredByMiddleware)
            .wrap(RequestLoggerMiddleware)
            .wrap(middleware::Compress::default())

            // 🩺 Root API — health check
            .route("/", web::get().to(healthcheck))

            // 👥 User endpoints
            .service(
                web::scope("/users")
                    .route("/create", web::post().to(create_user_handler))
                    .route("/check", web::post().to(check_user_active_handler))

                    // Development only
                    .route("", web::get().to(get_user_list_handler)),
            )
    })
    .bind(("0.0.0.0", 5500))?
    .run()
    .await
    .map_err(|e| {
        error!("❌ Server error: {}", e);
        e
    });

    // =====================
    // 🧹 Cleanup
    // =====================
    info!("Shutting down server...");
    drop(create_repo);
    drop(list_repo);
    drop(check_user_repo);

    info!("Closing RocksDB at {}", db_path);
    close_rocksdb(db);

    server_result
}
