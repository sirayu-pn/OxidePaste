mod models;
mod handlers;
mod db;
mod utils;

use axum::{routing::get, Router};
use dotenvy::dotenv;
use std::net::SocketAddr;
use tokio::time::{interval, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    println!("OxidePaste: Initializing database...");
    let pool = db::init_db().await;
    println!("OxidePaste: Database ready");

    // Background cleanup task
    let cleanup_pool = pool.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(300));
        loop {
            interval.tick().await;
            if let Ok(count) = db::cleanup_expired_pastes(&cleanup_pool).await {
                if count > 0 {
                    println!("OxidePaste: Cleaned up {} expired pastes", count);
                }
            }
        }
    });

    let app = Router::new()
        // Main routes
        .route("/", get(handlers::index).post(handlers::create_paste))
        .route("/:id", get(handlers::view_paste).post(handlers::verify_paste_password))
        .route("/:id/raw", get(handlers::view_raw))
        .route("/:id/delete", get(handlers::delete_paste))
        // Auth routes
        .route("/login", get(handlers::login_page).post(handlers::login))
        .route("/register", get(handlers::register_page).post(handlers::register))
        .route("/logout", get(handlers::logout))
        .route("/dashboard", get(handlers::dashboard))
        .route("/public", get(handlers::public_pastes))
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("OxidePaste: Server running at http://0.0.0.0:3000");
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}