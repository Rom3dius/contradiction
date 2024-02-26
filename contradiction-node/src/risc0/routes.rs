use bytes::{Buf, Bytes};
use http_body_util::BodyExt;
use std::collections::HashMap;
use crate::{risc0::models, handler::full, risc0::executor::execute_circuit, CLIENT};
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
    let receipt: Receipt = execute_circuit(&inputs)?;
    
    // insert receipt into database
    let uuid = insert(&pool, receipt.clone(), None).await?;

    let payload = responses::DefaultResponse {status_code: 201, text: uuid.to_string() };

    // send receipt to other nodes
    let node_payload = models::IncomingReceipt {uuid: uuid.to_string(), circuit: inputs, receipt: receipt};
    let nodes = sqlx::query_as::<_, responses::Node>(
        "SELECT address, port FROM nodes"
    )
    .fetch_all(&pool)
    .await?;
    for node in nodes {
        let url = format!("http://{}:{}/api/save-compute", node.address, node.port);
        let payload = serde_json::to_string(&node_payload)?;

        let response = CLIENT.post(url)
            .header("Content-Type", "application/json")
            .body(payload.clone()) // Clone payload for each request
            .send()
            .await;

        match response {
            Ok(resp) => {
                if !resp.status().is_success() {
                    log::warn!("Failed to send receipt to node. URL: {}, Status: {}", resp.url(), resp.status());
                }
            },
            Err(e) => {
                log::warn!("Error sending receipt to node: {}", e);
            }
        }
    }

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

async fn save_compute(req: Request<IncomingBody>, pool: SqlitePool) -> Result<Response<BoxBody>> {
    let incoming: models::IncomingReceipt = serde_json::from_reader(req.collect().await?.aggregate().reader())?;
    let id = models::fetch_circuit(&incoming.circuit).1;
    match incoming.receipt.verify(id) {
        Ok(_) => {
            let _ = insert(&pool, incoming.receipt, Some(incoming.uuid.clone())).await;
            let payload = responses::DefaultResponse {status_code: 201, text: incoming.uuid.to_string() };
            let response = Response::builder()
                .status(StatusCode::CREATED)
                .header(header::CONTENT_TYPE, "application/json")
                .body(full(serde_json::to_vec(&payload)?))?;

            Ok(response)
        },
        Err(_) => {
            let payload =  responses::DefaultResponse {status_code: 422, text: "Circuit receipt failed verifictation / an error during verification arose.".to_string()};
            let response = Response::builder()
                .status(StatusCode::UNPROCESSABLE_ENTITY)
                .header(header::CONTENT_TYPE, "application/json")
                .body(full(serde_json::to_vec(&payload)?))?;
            Ok(response)
        }
    }
}

pub async fn route_handler(req: Request<IncomingBody>, pool: SqlitePool) -> Result<Response<BoxBody>> {
    log::debug!("risc0 route handler checking request");
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/api/do-compute") => do_compute(req, pool).await,
        (&Method::GET, "/api/fetch-compute") => fetch_compute(req, pool).await,
        (&Method::POST, "/api/save-compute") => save_compute(req, pool).await,
        _ => {
            Err(anyhow::anyhow!("No matching GET condition"))
        },
    }
}