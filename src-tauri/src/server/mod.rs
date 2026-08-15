use crate::db::Database;
use crate::models::TempRequest;
use crate::template;
use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, Response, StatusCode},
    routing::any,
    Router,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tauri::Emitter;
use tokio::sync::oneshot;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;

pub struct ServerState {
    pub db: Arc<Database>,
    pub app_handle: tauri::AppHandle,
    pub temp_requests: Arc<Mutex<HashMap<String, TempRequest>>>,
}

pub async fn start_mock_server(
    db: Arc<Database>,
    app_handle: tauri::AppHandle,
    port: u16,
    cors_origins: Vec<String>,
    temp_requests: Arc<Mutex<HashMap<String, TempRequest>>>,
) -> Result<(tokio::task::JoinHandle<()>, oneshot::Sender<()>), String> {
    log::info!("start_mock_server called port={}", port);
    let state = Arc::new(ServerState { db, app_handle, temp_requests });

    let cors = if cors_origins.len() == 1 && cors_origins[0] == "*" {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        let origins: Vec<_> = cors_origins
            .iter()
            .filter_map(|o| o.parse::<axum::http::HeaderValue>().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(Any)
            .allow_headers(Any)
    };

    let app = Router::new()
        .fallback(any(handle_request))
        .layer(cors)
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024)) // 10MB limit
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind port {}: {}", port, e))?;
    log::info!("TcpListener bound to {}", addr);

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let handle = tokio::spawn(async move {
        let server = axum::serve(listener, app);
        let server = server.with_graceful_shutdown(async {
            let _ = shutdown_rx.await;
        });

        log::info!("Mock server started on {}", addr);
        if let Err(e) = server.await {
            log::error!("Server error: {}", e);
        }
        log::info!("Mock server stopped");
    });

    log::info!("spawned server task, returning handle");
    Ok((handle, shutdown_tx))
}

async fn handle_request(
    State(state): State<Arc<ServerState>>,
    request: Request,
) -> Response<Body> {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let query = request.uri().query().unwrap_or("").to_string();
    let headers = request.headers().clone();

    // Check for x-mock-example header (only used for db requests, not temp)
    let example_override = headers
        .get("x-mock-example")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Read request body for logging
    let body_bytes = match axum::body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => bytes::Bytes::new(),
    };
    let req_body = String::from_utf8_lossy(&body_bytes).to_string();

    // Run DB matching in a blocking thread to avoid blocking the async runtime
    let db_clone = state.db.clone();
    let method_clone = method.clone();
    let path_clone = path.clone();
    let db_match = tokio::task::spawn_blocking(move || {
        find_matching_db_request(&db_clone, &method_clone, &path_clone)
    })
    .await
    .unwrap_or(DbMatchResult::NoMatch);

    let response = match db_match {
        DbMatchResult::Matched(request_id, stored_path) => {
            log::info!("[MATCH] {} {} => DB request_id={}", method, path, request_id);
            // Database match - get example
            let path_params = extract_path_params(&stored_path, &path);

            // Run example lookup in a blocking thread
            let db_clone = state.db.clone();
            let request_id_clone = request_id.clone();
            let override_clone = example_override.clone();
            let example = tokio::task::spawn_blocking(move || {
                match override_clone.as_deref() {
                    Some(name) => find_example_by_name(&db_clone, &request_id_clone, name),
                    None => find_default_example(&db_clone, &request_id_clone),
                }
            })
            .await
            .unwrap_or(None);

            match example {
                Some((sc, body, response_headers)) => {
                    let mut builder = Response::builder().status(sc);

                    if let Ok(headers_arr) =
                        serde_json::from_str::<Vec<ResponseHeader>>(&response_headers)
                    {
                        for h in headers_arr {
                            if h.enabled {
                                builder = builder.header(&h.key, &h.value);
                            }
                        }
                    }

                    let mut context = HashMap::new();
                    for (k, v) in &path_params {
                        context.insert(format!("request.path.{}", k), v.clone());
                    }
                    let rendered_body = template::render_template(&body, &context);

                    builder
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(rendered_body))
                        .unwrap_or_else(|e| {
                            log::error!("Failed to build response: {}", e);
                            Response::new(Body::from("Internal Server Error"))
                        })
                }
                None => {
                    let mut resp = Response::new(Body::from("Not Found"));
                    *resp.status_mut() = StatusCode::NOT_FOUND;
                    resp
                }
            }
        }
        DbMatchResult::DisabledExact => {
            log::info!("[MATCH] {} {} => exact match DISABLED, returning 404", method, path);
            let mut resp = Response::new(Body::from("Not Found"));
            *resp.status_mut() = StatusCode::NOT_FOUND;
            resp
        }
        DbMatchResult::NoMatch => {
            // If no db match, try temp requests
            let temp_match = find_matching_temp_request(&state.temp_requests, &method, &path);
            if let Some(temp_req) = temp_match {
                log::info!("[MATCH] {} {} => TEMP request status={} tab_id={}", method, path, temp_req.status_code, temp_req.tab_id);
                // Temp request match - use stored response directly
                let path_params = extract_path_params(&temp_req.path, &path);

                let mut context = HashMap::new();
                for (k, v) in &path_params {
                    context.insert(format!("request.path.{}", k), v.clone());
                }
                let rendered_body = template::render_template(&temp_req.body, &context);

                let mut builder = Response::builder().status(temp_req.status_code);

                if let Ok(headers_arr) =
                    serde_json::from_str::<Vec<ResponseHeader>>(&temp_req.headers)
                {
                    for h in headers_arr {
                        if h.enabled {
                            builder = builder.header(&h.key, &h.value);
                        }
                    }
                }

                let content_type = match temp_req.body_type.as_str() {
                    "json" => "application/json",
                    "text" => "text/plain",
                    "html" => "text/html",
                    "xml" => "application/xml",
                    _ => "application/json",
                };

                builder
                    .header(header::CONTENT_TYPE, content_type)
                    .body(Body::from(rendered_body))
                    .unwrap_or_else(|e| {
                        log::error!("Failed to build temp response: {}", e);
                        Response::new(Body::from("Internal Server Error"))
                    })
            } else {
                log::info!("[MATCH] {} {} => 404 NOT FOUND", method, path);
                let mut resp = Response::new(Body::from("Not Found"));
                *resp.status_mut() = StatusCode::NOT_FOUND;
                resp
            }
        }
    };

    let status_code = response.status().as_u16();

    // Collect headers for logging
    let headers_json: Vec<(String, String)> = headers
        .iter()
        .filter_map(|(k, v)| {
            let key = k.as_str().to_string();
            let value = v.to_str().ok()?.to_string();
            Some((key, value))
        })
        .collect();

    // Emit log event to frontend
    let log_entry = serde_json::json!({
        "id": uuid::Uuid::new_v4().to_string(),
        "time": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        "method": method,
        "path": path,
        "status": status_code,
        "query": query,
        "headers": headers_json,
        "body": req_body,
    });
    let _ = state.app_handle.emit("server-log", log_entry);

    response
}

#[derive(Debug, serde::Deserialize)]
struct ResponseHeader {
    key: String,
    value: String,
    enabled: bool,
}

enum DbMatchResult {
    /// Found an enabled matching request
    Matched(String, String),
    /// An exact path match exists but is disabled — skip temp requests too
    DisabledExact,
    /// No match of any kind in database
    NoMatch,
}

/// Find matching request from database
fn find_matching_db_request(
    db: &Arc<Database>,
    method: &str,
    path: &str,
) -> DbMatchResult {
    let conn = match db.connection().lock() {
        Ok(c) => c,
        Err(_) => return DbMatchResult::NoMatch,
    };

    // Exact path match — collect ALL matching rows regardless of enabled state
    let mut stmt = match conn.prepare(
            "SELECT r.id, r.path, r.enabled, c.enabled FROM requests r
             JOIN collections c ON r.collection_id = c.id
             WHERE r.method = ?1 AND r.path = ?2"
    ) {
        Ok(s) => s,
        Err(_) => return DbMatchResult::NoMatch,
    };

    let exact_matches: Vec<(String, String, bool, bool)> = stmt
        .query_map([method, path], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get::<_, i32>(2)? != 0, row.get::<_, i32>(3)? != 0))
        })
        .into_iter()
        .flat_map(|r| r)
        .filter_map(|r| r.ok())
        .collect();

    for (id, stored_path, req_enabled, col_enabled) in &exact_matches {
        if *req_enabled && *col_enabled {
            log::info!("[DB_MATCH] exact match enabled: {} {} => {}", method, path, id);
            return DbMatchResult::Matched(id.clone(), stored_path.clone());
        }
    }

    if !exact_matches.is_empty() {
        // Exact path exists but ALL copies are disabled
        log::info!("[DB_MATCH] exact match ALL DISABLED: {} {}", method, path);
        return DbMatchResult::DisabledExact;
    }

    // No exact match at all — try path param matching
    log::info!("[DB_MATCH] no exact match for {} {}, trying path-param...", method, path);
    let mut stmt = match conn.prepare(
            "SELECT r.id, r.path FROM requests r
             JOIN collections c ON r.collection_id = c.id
             WHERE r.method = ?1 AND r.enabled = 1 AND c.enabled = 1"
    ) {
        Ok(s) => s,
        Err(_) => return DbMatchResult::NoMatch,
    };

    let paths: Vec<(String, String)> = stmt
        .query_map([method], |row| Ok((row.get(0)?, row.get(1)?)))
        .into_iter()
        .flat_map(|r| r)
        .filter_map(|r| r.ok())
        .collect();

    for (id, stored_path) in paths {
        if path_matches(&stored_path, path) {
            return DbMatchResult::Matched(id, stored_path);
        }
    }

    DbMatchResult::NoMatch
}

/// Find matching request from temp_requests in memory
fn find_matching_temp_request(
    temp_requests: &Arc<Mutex<HashMap<String, TempRequest>>>,
    method: &str,
    path: &str,
) -> Option<TempRequest> {
    let temp = temp_requests.lock().ok()?;

    // Key format: "METHOD:/path"
    let exact_key = format!("{}:{}", method.to_uppercase(), path.trim_start_matches('/'));
    if let Some(req) = temp.get(&exact_key) {
        return Some(req.clone());
    }

    // Path param match - iterate through temp requests
    for (_, req) in temp.iter() {
        if req.method.to_uppercase() == method.to_uppercase() && path_matches(&req.path, path) {
            return Some(req.clone());
        }
    }

    None
}

fn path_matches(pattern: &str, actual: &str) -> bool {
    let pattern_parts: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let actual_parts: Vec<&str> = actual.split('/').filter(|s| !s.is_empty()).collect();

    if pattern_parts.len() != actual_parts.len() {
        return false;
    }

    for (p, a) in pattern_parts.iter().zip(actual_parts.iter()) {
        if !p.starts_with(':') && p != a {
            return false;
        }
    }

    true
}

fn extract_path_params(pattern: &str, actual: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    let pattern_parts: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let actual_parts: Vec<&str> = actual.split('/').filter(|s| !s.is_empty()).collect();

    for (p, a) in pattern_parts.iter().zip(actual_parts.iter()) {
        if p.starts_with(':') {
            let key = p.trim_start_matches(':').to_string();
            params.insert(key, a.to_string());
        }
    }

    params
}

fn find_default_example(db: &Arc<Database>, request_id: &str) -> Option<(u16, String, String)> {
    let conn = db.connection().lock().ok()?;
    let mut stmt = conn
        .prepare("SELECT status_code, body, headers FROM examples WHERE request_id = ?1 AND is_default = 1 LIMIT 1")
        .ok()?;

    stmt.query_row([request_id], |row| {
        Ok((row.get::<_, i32>(0)? as u16, row.get(1)?, row.get(2)?))
    })
    .ok()
}

fn find_example_by_name(
    db: &Arc<Database>,
    request_id: &str,
    name: &str,
) -> Option<(u16, String, String)> {
    let conn = db.connection().lock().ok()?;
    let mut stmt = conn
        .prepare("SELECT status_code, body, headers FROM examples WHERE request_id = ?1 AND name = ?2 LIMIT 1")
        .ok()?;

    stmt.query_row([request_id, name], |row| {
        Ok((row.get::<_, i32>(0)? as u16, row.get(1)?, row.get(2)?))
    })
    .ok()
}
