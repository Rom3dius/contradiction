use sqlx::{Sqlite, migrate::MigrateDatabase, sqlite::SqlitePool, Row};
use anyhow::Result;
use serde::Serialize;
use uuid::Uuid;
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

        // create receipts table
        sqlx::query("CREATE TABLE IF NOT EXISTS receipts (
            uuid TEXT PRIMARY KEY,
            receipt TEXT NOT NULL
        );").execute(&pool).await.expect("Failed to create receipts table.");

    }
    SqlitePool::connect(&cfg.path).await.expect("Failed to create database connection")
}

pub async fn insert_receipt<T: Serialize>(pool: &SqlitePool, receipt: T) -> Result<Uuid> {
    // generate UUID and check for uniqueness
    let uuid = loop {
        let uuid = Uuid::new_v4();
        let exists = sqlx::query("SELECT EXISTS(SELECT 1 FROM receipts WHERE uuid = ?)")
            .bind(uuid.to_string())
            .fetch_one(pool)
            .await
            .expect("Failed to check for receipt existence")
            .get::<bool, _>(0);
        if !exists {
            break uuid;
        }
    };
    _ = sqlx::query("INSERT INTO receipts (uuid, receipt) VALUES (?, ?)")
        .bind(uuid.to_string())
        .bind(serde_json::to_string(&receipt).unwrap())
        .execute(pool)
        .await
        .map(|_| ());
    Ok(uuid)
}

pub async fn retrieve_receipt<T: for<'a> serde::Deserialize<'a>>(pool: &SqlitePool, uuid: &str) -> Result<T, sqlx::Error> {
    sqlx::query("SELECT receipt FROM receipts WHERE uuid = ?")
        .bind(uuid)
        .fetch_one(pool)
        .await
        .map(|row| serde_json::from_str(&row.get::<String, _>(0)).unwrap())
}