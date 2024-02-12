use sqlx::{Sqlite, migrate::MigrateDatabase, sqlite::SqlitePool};
use std::sync::Arc;
use crate::config;

pub async fn setup_database(cfg: &config::DB) -> Arc<SqlitePool> {
    if !Sqlite::database_exists(&cfg.path).await.unwrap_or(false) {
        println!("Creating database {}", &cfg.path);
        match Sqlite::create_database(&cfg.path).await {
            Ok(_) => println!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    }
    Arc::new(SqlitePool::connect(&cfg.path).await.expect("Failed to create database connection"))
}