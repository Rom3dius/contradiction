use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct TestJsonRequest {
    pub name: String,
    pub age: u16,
}