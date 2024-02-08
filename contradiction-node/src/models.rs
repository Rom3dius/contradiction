use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExamplePost {
    pub name: String,
    pub age: i32,
    pub email: String,
}