use bytes::{Buf, Bytes};
use http_body_util::{BodyExt, Full};
use hyper::{body::Incoming as IncomingBody, header, Method, Request, Response, StatusCode};
use sqlx::sqlite::SqlitePool;
use std::{arch::x86_64::_mm256_maskz_andnot_epi32, sync::Arc};

use anyhow::Result;

use contradiction_risc0 as risc0;

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

async fn do_compute(req: Request<IncomingBody>, pool: Arc<SqlitePool>) -> Result<Response<BoxBody>> {
    // deserialize circuit inputs
    let inputs: risc0::CircuitInputs = serde_json::from_reader(req.collect().await?.aggregate().reader())?; 

}

pub async fn route_handler(req: Request<IncomingBody>, pool: Arc<SqlitePool>) -> Result<Response<BoxBody>> {
    match request {
        (&Method::POST, "/api/do-compute") => do_compute(req, pool).await,
        _ => {
            // Return 404 not found response.
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(full(NOTFOUND))
                .unwrap())
        },
    };
}
