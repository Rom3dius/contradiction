use bytes::{Buf, Bytes};
use http_body_util::BodyExt;
use std::{str, collections::HashMap};
use crate::{risc0::models, handler::full, risc0::executor::execute_circuit};
use risc0_zkvm::Receipt;
use hyper::{body::Incoming as IncomingBody, header, Method, Request, Response, StatusCode};
use sqlx::sqlite::SqlitePool;
use anyhow::Result;
use crate::{models as responses, db::insert_receipt as insert, db::retrieve_receipt as retrieve};

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

async fn do_compute(req: Request<IncomingBody>, pool: SqlitePool) -> Result<Response<BoxBody>> {
    // deserialize circuit inputs
    let inputs: models::CircuitInputs = serde_json::from_reader(req.collect().await?.aggregate().reader())?;

    // execute and prove
    let receipt: Receipt = execute_circuit(inputs)?;
    
    // insert receipt into database
    let uuid = insert(&pool, receipt).await?;

    let payload = responses::DefaultResponse {status_code: 201, text: uuid.to_string() };

    // Use the ? operator to handle the Result returned by Response::builder()
    let response = Response::builder()
        .status(StatusCode::CREATED)
        .header(header::CONTENT_TYPE, "application/json")
        .body(full(serde_json::to_vec(&payload)?))?;
    
    Ok(response)
}

async fn fetch_compute(req: Request<IncomingBody>, pool: SqlitePool) -> Result<Response<BoxBody>> {
    // fetch UUID from query string
    let params: HashMap<String, String> = req
        .uri()
        .query()
        .map(|v| {
            url::form_urlencoded::parse(v.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_else(HashMap::new);
    let uuid: &String = match params.contains_key("uuid") {
        true => params.get("uuid").unwrap(),
        false => return Err(anyhow::anyhow!("No/malformed query string"))
    };
    let receipt: Receipt = retrieve(&pool, uuid).await?;
    let payload = responses::DefaultResponse {status_code: 200, text: serde_json::to_string(&receipt)? };
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(full(serde_json::to_vec(&payload)?))?;
    Ok(response)
}

pub async fn route_handler(req: Request<IncomingBody>, pool: SqlitePool) -> Result<Response<BoxBody>> {
    log::debug!("risc0 route handler checking request");
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/api/do-compute") => do_compute(req, pool).await,
        (&Method::GET, "/api/fetch-compute") => fetch_compute(req, pool).await,
        _ => {
            Err(anyhow::anyhow!("No matching GET condition"))
        },
    }
}