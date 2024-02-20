use bytes::Bytes;
use bytes::Buf;
use http_body_util::{BodyExt, Full};
use hyper::{body::Incoming as IncomingBody, header, Method, Request, Response, StatusCode};

use sqlx::sqlite::SqlitePool;

use crate::models;

use anyhow::Result;

#[cfg(feature="risc0")]
use crate::risc0::routes;

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

static INTERNAL_SERVER_ERROR: &[u8] = b"Internal Server Error";

async fn ping() -> Result<Response<BoxBody>> {
    let payload = models::NodeStatus {status: "online".to_string(), timestamp: chrono::Utc::now().naive_utc() };
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/text")
        .body(full(serde_json::to_vec(&payload)?))
        .expect("Failed to construct the response"))
}

async fn register_node(req: Request<IncomingBody>, db: SqlitePool) -> Result<Response<BoxBody>> {
    let node: models::Node = serde_json::from_reader(req.collect().await?.aggregate().reader())?;
    
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM nodes WHERE address = ? AND port = ?)"
    )
    .bind(&node.address)
    .bind(node.port as i32) 
    .fetch_one(&db)
    .await?;

    if !exists {
        sqlx::query(
            "INSERT INTO nodes (address, port) VALUES (?, ?)"
        )
        .bind(&node.address)
        .bind(node.port as i32) 
        .execute(&db)
        .await?;

        
        Ok(Response::builder()
            .status(StatusCode::CREATED)
            .body(full("Node registered successfully."))
            .expect("Failed to create node response")) 
    } else {
        // Respond that the node already exists
        Ok(Response::builder()
            .status(StatusCode::CONFLICT)
            .body(full("Node already exists."))
            .expect("Failed to create node response")) 
    }
}

async fn nodes(db: SqlitePool) -> Result<Response<BoxBody>> {
    let payload: Vec<models::Node> = sqlx::query_as::<_, models::Node>(
        "SELECT address, port FROM nodes"
    )
    .fetch_all(&db)
    .await?;

    let response = Response::builder()
        .status(StatusCode::CREATED)
        .header(header::CONTENT_TYPE, "application/json")
        .body(full(serde_json::to_vec(&payload)?))?;
    
    Ok(response)
}

pub async fn handle_request(req: Request<IncomingBody>, pool: SqlitePool) -> Result<Response<BoxBody>> {
    let request = (req.method(), req.uri().path());
    log::info!("Handling request: {} - {}", request.0.to_string(), request.1.to_string());

    let response: Result<Response<BoxBody>> = match request {
        (&Method::GET, "/ping") => ping().await,
        (&Method::POST, "/register_node") => register_node(req, pool).await,
        (&Method::GET, "/registered_nodes") => nodes(pool).await,
        _ => {
            #[cfg(feature="risc0")]
            routes::route_handler(req, pool).await
        },
    };

    // Error handler
    match response {
        Ok(res) => Ok(res),
        Err(err) => {
            if err.downcast_ref::<serde_json::Error>().is_some() {
                Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(full(INTERNAL_SERVER_ERROR))
                    .expect("Failed to construct the JSON error response"))
            } else if err.downcast_ref::<sqlx::Error>().is_some() {
                log::error!("Database error: {}", err);
                Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(full(INTERNAL_SERVER_ERROR))
                    .expect("Failed to construct the database error response"))
            } else {
                log::error!("Internal server error: {}", err);
                Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(full(INTERNAL_SERVER_ERROR))
                    .expect("Failed to construct the internal server error response"))
            }
        }
    }
}

pub fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}