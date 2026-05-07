use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub source_workspace: String,
    pub source_collection_id: Option<String>,
    pub created_at: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request {
    pub id: String,
    pub collection_id: String,
    pub name: String,
    pub method: String,
    pub path: String,
    pub description: Option<String>,
    pub headers: String,
    pub position: i32,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Example {
    pub id: String,
    pub request_id: String,
    pub name: String,
    pub is_default: bool,
    pub status_code: u16,
    pub headers: String,
    pub body: String,
    pub body_type: String,
    pub delay_ms: Option<i64>,
    pub match_rules: String,
    pub order_index: i32,
}

/// Lightweight request info used by example generator
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenRequest {
    pub id: String,
    pub collection_id: String,
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: Option<String>,
    pub body: Option<String>,
    pub position: i32,
}

/// Temporary request stored in memory for hot-reload
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TempRequest {
    pub tab_id: String,
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub headers: String,
    pub body: String,
    pub body_type: String,
    pub delay_ms: Option<i64>,
}

