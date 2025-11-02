use actix_web::{App, HttpServer, middleware, web};
use std::sync::Arc;

use crate::infrastructure::rocksdb::{init_rocksdb, close_rocksdb};
use crate::middleware::api_key::ApiKeyMiddleware;
use crate::middleware::powered_by::PoweredByMiddleware;

use crate::handler::create_user_handler::{create_user_handler, AppState as CreateUserAppState};
use crate::handler::get_user_list_handler::{get_user_list_handler, AppState as GetListAppState};
use crate::handler::check_user_active_handler::{check_user_active_handler, AppState as CheckUserActiveAppState};

use crate::services::create_user_service::CreateUserService;
use crate::services::get_user_list_service::GetUserListService;
use crate::services::check_user_active_service::CheckUserActiveService;

use crate::repositories::create_user_repository::CreateUserRepository;
use crate::repositories::get_user_list_repository::GetUserListRepository;
use crate::repositories::check_user_active_repository::CheckUserActiveRepository;

pub async fn run_server() -> std::io::Result<()> {
    // =====================
    // 🌱 Load Environment Variables
    // =====================
    dotenvy::dotenv().ok();

    // =====================
    // 🗄️ Database Initialization
    // =====================
    let db = init_rocksdb("./rocksdb-data/iotnet")
        .map_err(|e| {
            eprintln!("❌ Failed to initialize RocksDB: {}", e);
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to initialize RocksDB")
        })?;

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
    println!("🚀 Actix server running on http://0.0.0.0:5500");
    let server_result = HttpServer::new(move || {
        App::new()
            .app_data(create_state.clone())
            .app_data(list_state.clone())
            .app_data(check_state.clone())
            .wrap(ApiKeyMiddleware)
            .wrap(PoweredByMiddleware)
            .wrap(middleware::Compress::default())
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
        eprintln!("❌ Server error: {}", e);
        e
    });

    // =====================
    // 🧹 Cleanup
    // =====================
    drop(create_repo);
    drop(list_repo);
    drop(check_user_repo);
    println!("Shutting down... closing RocksDB instance");
    close_rocksdb(db);

    server_result
}
