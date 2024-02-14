use bytes::{Buf, Bytes};
use http_body_util::{BodyExt, Full};
use hyper::{body::Incoming as IncomingBody, header, Method, Request, Response, StatusCode};

use crate::models;

use sqlx::sqlite::SqlitePool;
use std::sync::Arc;

use anyhow::Result;

#[cfg(feature="risc0")]
use crate::risc_routes;

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

static INTERNAL_SERVER_ERROR: &[u8] = b"Internal Server Error";
static NOTFOUND: &[u8] = b"Not Found";

async fn ping() -> Result<Response<BoxBody>> {
    Ok(response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/text")
        .body("Pong!")
        .expect("Failed to construct the response"))
}

pub async fn handle_request(req: Request<IncomingBody>, pool: Arc<SqlitePool>) -> Result<Response<BoxBody>> {
    let request = (req.method(), req.uri().path());
    log::info!("Handling request: {} - {}", request.0.to_string(), request.1.to_string());

    if request.0 == &Method::GET && request.1 == "/ping" {
        ping();
    }

    #[cfg(feature="risc0")]
    let response = risc_routes::route_handler(req, pool);

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

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}