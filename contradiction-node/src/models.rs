use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExamplePost {
    pub name: String,
    pub age: i32,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultResponse {
    pub status_code: u8,
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct Node {
    pub address: String,
    pub port: u8,
    pub last_ping_at: chrono::NaiveDateTime,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NodeStatus {
    pub status: String,
    pub timestamp: chrono::NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    pub uuid: String,
}