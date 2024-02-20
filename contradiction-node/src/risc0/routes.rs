use bytes::{Buf, Bytes};
use http_body_util::BodyExt ;
use std::str;
use crate::{risc0::models, handler::full, risc0::executor::execute_circuit};
use risc0_zkvm::Receipt;
use hyper::{body::Incoming as IncomingBody, header, Method, Request, Response, StatusCode};
use sqlx::sqlite::SqlitePool;
use anyhow::Result;
use crate::models as responses;

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

async fn do_compute(req: Request<IncomingBody>, _pool: SqlitePool) -> Result<Response<BoxBody>> {
    // deserialize circuit inputs
    let inputs: models::CircuitInputs = serde_json::from_reader(req.collect().await?.aggregate().reader())?;

    // execute and prove
    let receipt: Receipt = execute_circuit(inputs)?;
    

    let journal = str::from_utf8(&receipt.journal.bytes)?;
    let payload = responses::DefaultResponse {status_code: 201, text: journal.to_string() };
    
    // Use the ? operator to handle the Result returned by Response::builder()
    let response = Response::builder()
        .status(StatusCode::CREATED)
        .header(header::CONTENT_TYPE, "application/json")
        .body(full(serde_json::to_vec(&payload)?))?;
    
    Ok(response)

}

pub async fn route_handler(req: Request<IncomingBody>, pool: SqlitePool) -> Result<Response<BoxBody>> {
    log::debug!("risc0 route handler checking request");
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/api/do-compute") => do_compute(req, pool).await,
        _ => {
            // Return 404 not found response.
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(full("NOT FOUND"))
                .unwrap())
        },
    }
}