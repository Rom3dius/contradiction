use hyper::{Request, Response, StatusCode, header, Method, body::Bytes, body::Body};
use anyhow::Result;
use http_body_util::Full;
use serde::{Deserialize, Serialize};

use crate::models::ExamplePost;
use sqlx::SqlitePool;
use std::sync::Arc;
// use http_body_util::Full;

/// DEFINE ROUTES HERE ///

pub async fn api_post_example(pool: Arc<SqlitePool>, req: hyper::Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>> {
    let request_json = req.into_body();
    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}

/// REQUEST HANDLER ///

pub async fn handle_request(req: hyper::Request<hyper::body::Incoming>, pool: Arc<SqlitePool>) -> Result<Response<Full<Bytes>>> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/api/example") => api_post_example(pool, req).await,
        _ => {
            Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
        }
    }
}