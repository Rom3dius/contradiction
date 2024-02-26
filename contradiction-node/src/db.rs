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

pub async fn insert_receipt<T: Serialize>(pool: &SqlitePool, receipt: T, uuid: Option<String>) -> Result<Uuid> {
    let uuid = match uuid {
        Some(u) => {
            let exists = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM receipts WHERE uuid = ?)")
                .bind(&u)
                .fetch_one(pool)
                .await?;

            if exists {
                // If the UUID exists, return an error
                return Err(anyhow::anyhow!("UUID already exists"));
            }
            Uuid::parse_str(&u).map_err(|_| anyhow::anyhow!("Invalid UUID format"))?
        },
        None => {
            // Generate a new unique UUID
            loop {
                let new_uuid = Uuid::new_v4();
                let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM receipts WHERE uuid = ?)")
                    .bind(new_uuid.to_string())
                    .fetch_one(pool)
                    .await?;

                if !exists {
                    break new_uuid;
                }
            }
        }
    };

    // Insert the receipt with the UUID
    sqlx::query("INSERT INTO receipts (uuid, receipt) VALUES (?, ?)")
        .bind(uuid.to_string())
        .bind(serde_json::to_string(&receipt).expect("Failed to serialize receipt"))
        .execute(pool)
        .await?;

    Ok(uuid)
}

pub async fn retrieve_receipt<T: for<'a> serde::Deserialize<'a>>(pool: &SqlitePool, uuid: &str) -> Result<T, sqlx::Error> {
    sqlx::query("SELECT receipt FROM receipts WHERE uuid = ?")
        .bind(uuid)
        .fetch_one(pool)
        .await
        .map(|row| serde_json::from_str(&row.get::<String, _>(0)).unwrap())
}