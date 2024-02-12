use bytes::{Buf, Bytes};
use http_body_util::{BodyExt, Full};
use hyper::{body::Incoming as IncomingBody, header, Method, Request, Response, StatusCode};

use crate::models;

use sqlx::sqlite::SqlitePool;
use std::sync::Arc;

use anyhow::Result;

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

static INTERNAL_SERVER_ERROR: &[u8] = b"Internal Server Error";
static NOTFOUND: &[u8] = b"Not Found";

async fn api_post_response(req: Request<IncomingBody>) -> Result<Response<BoxBody>> {
    let whole_body = req.collect().await?.aggregate();
    let mut data: models::ExamplePost = serde_json::from_reader(whole_body.reader())?;
    data.name = "test_value".to_string();
    
    // Serialize your response object to JSON.
    let json = serde_json::to_string(&data)?;
    
    let response = models::DefaultResponse { status_code: 200, text: "test_value".to_string() };
    let serialized_response = serde_json::to_vec(&response)?;
    
    let body = Full::new(Bytes::from(serialized_response))?.boxed();

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(body)
        .expect("Failed to construct the response");

    Ok(response)
}

pub async fn handle_request(req: Request<IncomingBody>, pool: Arc<SqlitePool>) -> Result<Response<BoxBody>> {
    let response = (req.method(), req.uri().path());
    log::info!("Handling request: {} - {}", response.0.to_string(), response.1.to_string());
    let response = match response {
        (&Method::POST, "/json_api") => api_post_response(req).await,
        _ => {
            // Return 404 not found response.
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(full(NOTFOUND))
                .unwrap())
        }
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

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}