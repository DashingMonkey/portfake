use crate::db::examples;
use crate::models::Example;
use crate::{lock_or_recover, AppState};
use rusqlite::params;
use std::sync::Mutex;

#[tauri::command]
pub fn get_examples(
    state: tauri::State<'_, Mutex<AppState>>,
    request_id: String,
) -> Result<Vec<Example>, String> {
    let app_state = lock_or_recover(&state);
    examples::get_examples_by_request(&app_state.db, &request_id)
}

#[tauri::command]
pub fn update_example(
    state: tauri::State<'_, Mutex<AppState>>,
    example_id: String,
    status_code: u16,
    headers: String,
    body: String,
    body_type: String,
    delay_ms: Option<i64>,
) -> Result<(), String> {
    let app_state = lock_or_recover(&state);
    let conn = app_state.db.connection().lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE examples SET status_code = ?1, headers = ?2, body = ?3, body_type = ?4, delay_ms = ?5 WHERE id = ?6",
        params![status_code as i32, headers, body, body_type, delay_ms, example_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}
