mod config;
mod models;
mod handler;
mod db;

#[cfg(feature="risc0")]
mod risc0;

use tokio::net::TcpListener;
use tokio::signal;
use hyper::service::service_fn;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo as io;
use std::str::FromStr;
use reqwest::Client;

#[macro_use]
extern crate lazy_static;
use std::time::SystemTime;

use fern;

use anyhow::Result;

lazy_static! {
    static ref START_TIME: SystemTime = SystemTime::now(); 
}

lazy_static! {
    static ref CLIENT: Client = Client::new();
}

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

    // Handle nodes
    if let Some(nodes) = &config.nodes {
        for node in nodes {
            // Construct the URI
            let uri = format!("http://{}:{}", node.address, node.port);
            
            // Validate the URI
            match hyper::Uri::from_str(&uri) {
                Ok(_) => {
                    // Insert the node into the database
                    let _ = sqlx::query(
                        "
                        INSERT INTO nodes (address, port)
                        VALUES ($1, $2)
                        ",
                    )
                    .bind(&node.address)
                    .bind(node.port)
                    .execute(&pool)
                    .await;
                },
                Err(e) => {
                    log::error!("Invalid URI for node: {}:{}. Error: {}", node.address, node.port, e);
                }
            }
        }
    }

    // Set up a TCP listener
    let addr = config.socket_address();
    let listener = TcpListener::bind(addr).await?;
    log::info!("Listening on: {}", addr);

    // Run the server!
    let server_pool = pool.clone();
    let server = tokio::spawn(async move {
        loop {
            let (stream, _) = match listener.accept().await {
                Ok(conn) => conn,
                Err(e) => {
                    log::error!("Failed to accept connection: {}", e);
                    continue;
                }
            };

            let server_pool = server_pool.clone();
            tokio::spawn(async move {
                let service = service_fn(move |req| handler::handle_request(req, server_pool.clone()));
                let io = io::new(stream);

                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                    log::error!("Failed to serve connection: {:?}", err);
                }
            });
        }
    });

    let update_nodes = tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            let db = pool.clone();
            let nodes: Vec<models::Node> = sqlx::query_as::<_, models::Node>(
                    "SELECT address, port FROM nodes"
                )
                .fetch_all(&db)
                .await.expect("Failed to fetch nodes from database.");
            for node in nodes {
                let node_status = CLIENT.get(&format!("http://{}:{}/ping", node.address, node.port)).send().await;
                match node_status {
                    Ok(response) => {
                        if response.status().is_success() {
                            log::info!("Node {}:{} is alive.", node.address, node.port);
                        } else {
                            log::info!("Node {}:{} is dead.", node.address, node.port);
                            sqlx::query("DELETE FROM nodes WHERE address = ? AND port = ?")
                                .bind(node.address)
                                .bind(node.port)
                                .execute(&db)
                                .await
                                .expect("Failed to delete dead node.");
                        }
                    },
                    Err(_e) => {
                        log::info!("Node {}:{} is dead/url malformed.", node.address, node.port);
                        sqlx::query("DELETE FROM nodes WHERE address = ? AND port = ?")
                            .bind(node.address)
                            .bind(node.port)
                            .execute(&db)
                            .await
                            .expect("Failed to delete dead node.");
                    }
                }
            }
        }
    });

    // Wait for CTRL+C signal
    signal::ctrl_c().await.expect("Failed to listen for ctrl+c signal");

    log::info!("Shutdown signal received, shutting down.");

    server.abort();
    update_nodes.abort();

    Ok(())
}
