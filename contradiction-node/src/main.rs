mod config;
mod models;
mod routes;
mod db;

use tokio::net::TcpListener;
use tokio::signal;
use hyper::service::service_fn;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo as io;

use fern;

use anyhow::Result;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load config
    let config = config::Config::read_config();

    // Setup logging
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339(std::time::SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(config.log.level)
        .chain(if config.log.stdout {
            Box::new(std::io::stdout()) as Box<dyn std::io::Write + Send>
        } else {
            Box::new(std::io::sink()) // If stdout is false, don't log to stdout
        })
        .chain(fern::log_file(&config.log.file_output)?)
        .apply()?;

    // Setup database
    let pool = db::setup_database(&config.db).await;
    log::debug!("Created database pool.");

    // Set up a TCP listener
    let addr = config.socket_address();
    let listener = TcpListener::bind(addr).await?;
    log::info!("Listening on: {}", addr);

    // Run the server!
    let server = tokio::spawn(async move {
        loop {
            let (stream, _) = match listener.accept().await {
                Ok(conn) => conn,
                Err(e) => {
                    log::error!("Failed to accept connection: {}", e);
                    continue;
                }
            };

            let pool_clone = pool.clone();
            tokio::spawn(async move {
                let service = service_fn(move |req| routes::handle_request(req, Arc::clone(&pool_clone)));
                let io = io::new(stream);

                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                    log::error!("Failed to serve connection: {:?}", err);
                }
            });
        }
    });

    // Wait for CTRL+C signal
    signal::ctrl_c().await.expect("Failed to listen for ctrl+c signal");

    log::info!("Shutdown signal received, shutting down.");

    server.abort();

    Ok(())
}
