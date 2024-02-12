use serde::Serialize;

#[derive(Serialize)]
pub struct DefaultResponse {
    pub status_code: u16,
    pub text: String,
}