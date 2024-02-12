use bytes::{Buf, Bytes};
use http_body_util::{BodyExt, Full};
use hyper::{body::Incoming as IncomingBody, header, Method, Request, Response, StatusCode};

use contradiction_models::{requests, responses};

use sqlx::sqlite::SqlitePool;
use std::sync::Arc;

use anyhow::Result;

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

static INTERNAL_SERVER_ERROR: &[u8] = b"Internal Server Error";
static NOTFOUND: &[u8] = b"Not Found";

async fn api_post_response(req: Request<IncomingBody>) -> Result<Response<BoxBody>> {
    let whole_body = req.collect().await?.aggregate();
    let mut data: requests::TestJsonRequest = serde_json::from_reader(whole_body.reader())?;
    data.name = "test_value".to_string();
    let json = serde_json::to_string(&data)?;
    let response = responses::DefaultResponse { status_code: 200, text: "test_value".to_string() };
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(serde_json::to_vec(&response)?);
    Ok(response)
}

pub async fn handle_request(req: Request<IncomingBody>, pool: Arc<SqlitePool>) -> Result<Response<BoxBody>> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/json_api") => api_post_response(req).await,
        _ => {
            // Return 404 not found response.
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(full(NOTFOUND))
                .unwrap())
        }
    }
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}