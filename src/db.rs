use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::env;

pub async fn init_db() -> SqlitePool {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./oxide-paste.db?mode=rwc".to_string());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Create pastes table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS pastes (
            id TEXT PRIMARY KEY,
            content TEXT NOT NULL,
            language TEXT,
            password_hash TEXT,
            expires_at DATETIME,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            view_count INTEGER NOT NULL DEFAULT 0,
            user_id INTEGER
        )"
    )
    .execute(&pool)
    .await
    .expect("Failed to create pastes table");

    // Create users table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(&pool)
    .await
    .expect("Failed to create users table");

    // Add user_id column if not exists (for existing databases)
    let _ = sqlx::query("ALTER TABLE pastes ADD COLUMN user_id INTEGER")
        .execute(&pool)
        .await;

    // Create indexes
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_pastes_expires_at ON pastes(expires_at)")
        .execute(&pool)
        .await;

    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_pastes_user_id ON pastes(user_id)")
        .execute(&pool)
        .await;

    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_username ON users(username)")
        .execute(&pool)
        .await;

    pool
}

pub async fn cleanup_expired_pastes(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM pastes WHERE expires_at IS NOT NULL AND expires_at < datetime('now')")
        .execute(pool)
        .await?;
    
    Ok(result.rows_affected())
}