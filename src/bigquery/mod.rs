mod client;
mod queries;

pub use client::BigQueryClient;
use queries::*;

use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct BigQueryConfig {
    pub project_id: String,
    pub credentials_path: String,
}

#[derive(Debug)]
pub enum BigQueryError {
    Auth(String),
    Query(String),
    Parse(String),
}

impl std::fmt::Display for BigQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BigQueryError::Auth(msg) => write!(f, "BigQuery authentication error: {}", msg),
            BigQueryError::Query(msg) => write!(f, "BigQuery query error: {}", msg),
            BigQueryError::Parse(msg) => write!(f, "BigQuery parse error: {}", msg),
        }
    }
}

impl Error for BigQueryError {}
