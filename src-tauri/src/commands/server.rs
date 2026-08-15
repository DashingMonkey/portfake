use crate::models::TempRequest;
use crate::{lock_or_recover, AppState};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct ServerStatus {
    pub running: bool,
    pub port: u16,
}

#[tauri::command]
pub async fn start_server(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
    port: u16,
    cors_origins: Vec<String>,
    drafts: Vec<TempRequest>,
) -> Result<u16, String> {
    log::info!("start_server called with port={} cors_origins={:?} drafts_count={}", port, cors_origins, drafts.len());

    if port < 1024 {
        log::warn!("Port {} is in the privileged range (0-1023), this may require admin/root privileges", port);
    }

    // Build temp_requests map and store in shared Arc
    let temp_requests_map: HashMap<String, TempRequest> = drafts
        .into_iter()
        .map(|draft| {
            let key = format!("{}:{}", draft.method.to_uppercase(), draft.path.trim_start_matches('/'));
            log::info!("Loaded temp request: {}", key);
            (key, draft)
        })
        .collect();

    // Update the shared temp_requests Arc (same instance used by server)
    {
        let mut temp = lock_or_recover(&state.temp_requests);
        *temp = temp_requests_map;
    }

    // Stop existing server if running
    let old_handle = {
        if let Some(tx) = lock_or_recover(&state.shutdown_tx).take() {
            log::info!("Stopping existing server...");
            let _ = tx.send(());
        }
        lock_or_recover(&state.server_handle).take()
    };

    // Await old server to fully stop before rebinding (prevents port race)
    if let Some(handle) = old_handle {
        log::info!("Waiting for old server to stop...");
        let _ = handle.await;
        log::info!("Old server stopped");
    }

    let db = state.db.clone();
    let temp_requests = state.temp_requests.clone();

    let (handle, shutdown_tx) =
        crate::server::start_mock_server(db, app_handle, port, cors_origins, temp_requests).await?;
    log::info!("start_mock_server returned successfully");

    {
        *lock_or_recover(&state.server_handle) = Some(handle);
        *lock_or_recover(&state.shutdown_tx) = Some(shutdown_tx);
        *lock_or_recover(&state.server_port) = port;
    }

    log::info!("Server started on port {}", port);
    Ok(port)
}

#[tauri::command]
pub async fn stop_server(state: tauri::State<'_, AppState>) -> Result<(), String> {
    if let Some(tx) = lock_or_recover(&state.shutdown_tx).take() {
        let _ = tx.send(());
    }
    let handle = lock_or_recover(&state.server_handle).take();
    if let Some(handle) = handle {
        let _ = handle.await;
        log::info!("Server stopped");
    }

    // Clear temp requests from memory
    {
        let mut temp = lock_or_recover(&state.temp_requests);
        temp.clear();
        log::info!("Cleared temp requests from memory");
    }

    Ok(())
}

#[tauri::command]
pub fn get_server_status(state: tauri::State<'_, AppState>) -> Result<ServerStatus, String> {
    let running = lock_or_recover(&state.server_handle).is_some();
    let port = *lock_or_recover(&state.server_port);

    Ok(ServerStatus { running, port })
}

#[tauri::command]
pub fn sync_temp_request(
    state: tauri::State<'_, AppState>,
    draft: TempRequest,
) -> Result<(), String> {
    let mut temp = lock_or_recover(&state.temp_requests);
    let key = format!("{}:{}", draft.method.to_uppercase(), draft.path.trim_start_matches('/'));
    temp.insert(key.clone(), draft);
    log::info!("Synced temp request: {}", key);
    Ok(())
}

#[tauri::command]
pub fn remove_temp_request(
    state: tauri::State<'_, AppState>,
    tab_id: String,
) -> Result<(), String> {
    let mut temp = lock_or_recover(&state.temp_requests);

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
