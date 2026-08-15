use crate::db::examples;
use crate::models::Example;
use crate::AppState;

#[tauri::command]
pub fn get_examples(
    state: tauri::State<'_, AppState>,
    request_id: String,
) -> Result<Vec<Example>, String> {
    examples::get_examples_by_request(&state.db, &request_id)
}

#[tauri::command]
pub fn update_example(
    state: tauri::State<'_, AppState>,
    example_id: String,
    status_code: u16,
    headers: String,
    body: String,
    body_type: String,
    delay_ms: Option<i64>,
) -> Result<(), String> {
    examples::update_example(&state.db, &example_id, status_code, &headers, &body, &body_type, delay_ms)
}
