mod config;
mod models;
mod routes;

use sqlx::sqlite::SqlitePool;
use std::sync::Arc;
use tokio::net::TcpListener;
use hyper::service::service_fn;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo as io;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Load config
    let config = config::Config::read_config();

    // Initialize SQLite connection pool
    let pool = Arc::new(SqlitePool::connect(&config.db.path).await.expect("Failed to create database connection"));

    // Set up a TCP listener
    let addr = config.socket_address();
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on: {}", addr);

    // Accept incoming connections and serve them
    loop {
        let (stream, _) = listener.accept().await?;
        let io = io::new(stream);
        let pool_clone = pool.clone();

        // Spawn a new task for each connection
        tokio::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(move |req| {
                    let pool = pool_clone.clone();
                    async move {
                        routes::handle_request(req, pool).await
                    }
                }))
                .await
            {
                eprintln!("server error: {}", err);
            }
        });
    }
}
