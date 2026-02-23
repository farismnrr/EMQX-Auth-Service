mod dtos;
mod entities;
mod handler;
mod infrastructure;
mod middleware;
mod repositories;
mod server;
mod services;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    server::run_server().await
}
