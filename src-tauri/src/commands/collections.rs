use crate::db::collections;
use crate::models::Collection;
use crate::AppState;

#[tauri::command]
pub fn get_collections(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Collection>, String> {
    collections::get_collections(&state.db)
}

#[tauri::command]
pub fn create_collection(
    state: tauri::State<'_, AppState>,
    name: String,
) -> Result<Collection, String> {
    let id = uuid::Uuid::new_v4().to_string();
    collections::create_collection(&state.db, &id, &name, "manual", None)
}

#[tauri::command]
pub fn delete_collection(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    collections::delete_collection(&state.db, &id)
}

#[tauri::command]
pub fn rename_collection(
    state: tauri::State<'_, AppState>,
    id: String,
    name: String,
) -> Result<Collection, String> {
    collections::rename_collection(&state.db, &id, &name)
}

#[tauri::command]
pub fn set_collection_enabled(
    state: tauri::State<'_, AppState>,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    collections::set_collection_enabled(&state.db, &id, enabled)
}
