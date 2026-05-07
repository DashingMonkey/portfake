use crate::db::collections;
use crate::models::Collection;
use crate::{lock_or_recover, AppState};
use std::sync::Mutex;

#[tauri::command]
pub fn get_collections(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<Collection>, String> {
    let app_state = lock_or_recover(&state);
    collections::get_collections(&app_state.db)
}

#[tauri::command]
pub fn create_collection(
    state: tauri::State<'_, Mutex<AppState>>,
    name: String,
) -> Result<Collection, String> {
    let app_state = lock_or_recover(&state);
    let id = uuid::Uuid::new_v4().to_string();
    collections::create_collection(&app_state.db, &id, &name, "manual", None)
}

#[tauri::command]
pub fn delete_collection(
    state: tauri::State<'_, Mutex<AppState>>,
    id: String,
) -> Result<(), String> {
    let app_state = lock_or_recover(&state);
    collections::delete_collection(&app_state.db, &id)
}

#[tauri::command]
pub fn rename_collection(
    state: tauri::State<'_, Mutex<AppState>>,
    id: String,
    name: String,
) -> Result<Collection, String> {
    let app_state = lock_or_recover(&state);
    collections::rename_collection(&app_state.db, &id, &name)
}

#[tauri::command]
pub fn set_collection_enabled(
    state: tauri::State<'_, Mutex<AppState>>,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    let app_state = lock_or_recover(&state);
    collections::set_collection_enabled(&app_state.db, &id, enabled)
}
