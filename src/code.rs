use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CodeRequest {
    pub language: String,
    pub source_code: String,
    pub input: String,
    pub time_limit: u8,
}

#[derive(Debug, Serialize)]
pub struct CodeResponse {
    pub status: String,
    pub message: String,
}
