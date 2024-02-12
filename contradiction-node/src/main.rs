mod config;
mod models;
mod routes;
mod db;

use tokio::net::TcpListener;
use hyper::service::service_fn;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo as io;

use anyhow::Result;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load config
    let config = config::Config::read_config();

    // Setup database
    let pool = db::setup_database(&config.db).await;

    // Set up a TCP listener
    let addr = config.socket_address();
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on: {}", addr);

    // Accept incoming connections and serve them
    loop {
        let (stream, _) = listener.accept().await?;
        let pool_clone = pool.clone(); // Clone the pool for each iteration


        tokio::spawn(async move {
            let pool = Arc::clone(&pool_clone);
            let service = service_fn(move |req| routes::handle_request(req, Arc::clone(&pool)));
            let io = io::new(stream);

            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                println!("Failed to serve connection: {:?}", err);
            }
        });
    }
}
