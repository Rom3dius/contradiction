use sqlx::{Sqlite, migrate::MigrateDatabase, sqlite::SqlitePool};
use crate::config;

pub async fn setup_database(cfg: &config::DB) -> SqlitePool {
    if !Sqlite::database_exists(&cfg.path).await.unwrap_or(false) {
        println!("Creating database {}", &cfg.path);
        Sqlite::create_database(&cfg.path).await.expect("Failed to create database.");
        let pool = SqlitePool::connect(&cfg.path).await.expect("Failed to create database connection.");

        // create nodes table
        sqlx::query("CREATE TABLE IF NOT EXISTS nodes (
            address TEXT NOT NULL,
            port INTEGER NOT NULL,
            last_ping_at INTEGER,
            PRIMARY KEY (address, port)
        );").execute(&pool).await.expect("Failed to create nodes table.");

    }
    SqlitePool::connect(&cfg.path).await.expect("Failed to create database connection")
}