use crate::models::TempRequest;
use crate::{lock_or_recover, AppState};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize)]
pub struct ServerStatus {
    pub running: bool,
    pub port: u16,
}

#[tauri::command]
pub async fn start_server(
    state: tauri::State<'_, Mutex<AppState>>,
    app_handle: tauri::AppHandle,
    port: u16,
    cors_origins: Vec<String>,
    drafts: Vec<TempRequest>,
) -> Result<u16, String> {
    log::info!("start_server called with port={} cors_origins={:?} drafts_count={}", port, cors_origins, drafts.len());

    if port < 1024 {
        log::warn!("Port {} is in the privileged range (0-1023), this may require admin/root privileges", port);
    }

    // Build temp_requests map and clear old one
    let temp_requests: HashMap<String, TempRequest> = drafts
        .into_iter()
        .map(|draft| {
            let key = format!("{}:{}", draft.method.to_uppercase(), draft.path.trim_start_matches('/'));
            log::info!("Loaded temp request: {}", key);
            (key, draft)
        })
        .collect();

    let temp_requests = Arc::new(Mutex::new(temp_requests));
    let temp_requests_for_server = temp_requests.clone();

    let db = {
        let app_state = lock_or_recover(&state);
        log::info!("Acquired app_state lock");

        // Stop existing server if running
        if let Some(tx) = app_state.shutdown_tx.lock().ok().and_then(|mut g| g.take()) {
            log::info!("Stopping existing server...");
            let _ = tx.send(());
        }
        if let Some(handle) = app_state.server_handle.lock().ok().and_then(|mut g| g.take()) {
            drop(handle);
            log::info!("Dropped existing server handle");
        }

        // Update temp_requests in AppState
        if let Ok(mut temp) = app_state.temp_requests.lock() {
            *temp = temp_requests_for_server.lock().unwrap().clone();
        }

        log::info!("Cloned db and dropped app_state");
        app_state.db.clone()
    };

    let (handle, shutdown_tx) =
        crate::server::start_mock_server(db, app_handle, port, cors_origins, temp_requests).await?;
    log::info!("start_mock_server returned successfully");

    {
        let app_state = lock_or_recover(&state);
        *lock_or_recover(&app_state.server_handle) = Some(handle);
        *lock_or_recover(&app_state.shutdown_tx) = Some(shutdown_tx);
        *lock_or_recover(&app_state.server_port) = port;
    }

    log::info!("Server started on port {}", port);
    Ok(port)
}

#[tauri::command]
pub async fn stop_server(state: tauri::State<'_, Mutex<AppState>>) -> Result<(), String> {
    let app_state = lock_or_recover(&state);

    if let Some(tx) = app_state.shutdown_tx.lock().ok().and_then(|mut g| g.take()) {
        let _ = tx.send(());
    }
    if let Some(handle) = app_state.server_handle.lock().ok().and_then(|mut g| g.take()) {
        drop(handle);
        log::info!("Server stopped");
    }

    // Clear temp requests from memory
    if let Ok(mut temp) = app_state.temp_requests.lock() {
        temp.clear();
        log::info!("Cleared temp requests from memory");
    }

    Ok(())
}

#[tauri::command]
pub fn get_server_status(state: tauri::State<'_, Mutex<AppState>>) -> Result<ServerStatus, String> {
    let app_state = lock_or_recover(&state);
    let running = lock_or_recover(&app_state.server_handle).is_some();
    let port = *lock_or_recover(&app_state.server_port);

    Ok(ServerStatus { running, port })
}

#[tauri::command]
pub fn sync_temp_request(
    state: tauri::State<'_, Mutex<AppState>>,
    draft: TempRequest,
) -> Result<(), String> {
    let app_state = lock_or_recover(&state);
    let mut temp = app_state.temp_requests.lock().map_err(|e| e.to_string())?;
    let key = format!("{}:{}", draft.method.to_uppercase(), draft.path.trim_start_matches('/'));
    temp.insert(key.clone(), draft);
    log::info!("Synced temp request: {}", key);
    Ok(())
}

#[tauri::command]
pub fn remove_temp_request(
    state: tauri::State<'_, Mutex<AppState>>,
    tab_id: String,
) -> Result<(), String> {
    let app_state = lock_or_recover(&state);
    let mut temp = app_state.temp_requests.lock().map_err(|e| e.to_string())?;

    // Find and remove by tab_id
    let key_to_remove = temp.iter()
        .find(|(_, v)| v.tab_id == tab_id)
        .map(|(k, _)| k.clone());

    if let Some(key) = key_to_remove {
        temp.remove(&key);
        log::info!("Removed temp request: {}", key);
    }

    Ok(())
}
