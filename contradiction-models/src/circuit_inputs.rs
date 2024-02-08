use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct MyModel {
    pub id: i32,
    pub data: String,
    // other fields...
}
