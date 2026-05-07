use crate::db::{examples, requests};
use crate::example_gen;
use crate::models::{GenRequest, Request};
use crate::{lock_or_recover, AppState};
use std::sync::Mutex;

#[tauri::command]
pub fn get_requests(
    state: tauri::State<'_, Mutex<AppState>>,
    collection_id: String,
) -> Result<Vec<Request>, String> {
    let app_state = lock_or_recover(&state);
    requests::get_requests_by_collection(&app_state.db, &collection_id)
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn create_request(
    state: tauri::State<'_, Mutex<AppState>>,
    collection_id: String,
    name: String,
    method: String,
    path: String,
    example_status_code: Option<u16>,
    example_headers: Option<String>,
    example_body: Option<String>,
    example_body_type: Option<String>,
    example_delay_ms: Option<i64>,
) -> Result<Request, String> {
    let app_state = lock_or_recover(&state);
    let id = uuid::Uuid::new_v4().to_string();
    let request = requests::create_request(&app_state.db, &id, &collection_id, &name, &method, &path, "[]")?;

    // Create example with user-provided data or auto-generated defaults
    let ping_req = GenRequest {
        id: id.clone(),
        collection_id: collection_id.clone(),
        name: name.clone(),
        method: method.clone(),
        url: path.clone(),
        headers: Some("[]".to_string()),
        body: None,
        position: request.position,
    };

    let gen = example_gen::generate_example(&ping_req);
    let example_id = uuid::Uuid::new_v4().to_string();

    let example_params = examples::CreateExampleParams {
        db: app_state.db.clone(),
        id: example_id,
        request_id: id,
        name: gen.name,
        is_default: true,
        status_code: example_status_code.unwrap_or(gen.status_code),
        headers: example_headers.unwrap_or(gen.headers),
        body: example_body.unwrap_or(gen.body),
        body_type: example_body_type.unwrap_or(gen.body_type),
        delay_ms: example_delay_ms.or(gen.delay_ms),
    };
    examples::create_example(example_params)?;

    Ok(request)
}

#[tauri::command]
pub fn delete_request(
    state: tauri::State<'_, Mutex<AppState>>,
    request_id: String,
) -> Result<(), String> {
    let app_state = lock_or_recover(&state);
    requests::delete_request(&app_state.db, &request_id)
}

#[tauri::command]
pub fn update_request(
    state: tauri::State<'_, Mutex<AppState>>,
    request_id: String,
    collection_id: String,
    name: String,
    method: String,
    path: String,
    headers: String,
) -> Result<(), String> {
    let app_state = lock_or_recover(&state);
    requests::update_request(&app_state.db, &request_id, &collection_id, &name, &method, &path, &headers)
}

#[tauri::command]
pub fn set_request_enabled(
    state: tauri::State<'_, Mutex<AppState>>,
    request_id: String,
    enabled: bool,
) -> Result<(), String> {
    let app_state = lock_or_recover(&state);
    requests::set_request_enabled(&app_state.db, &request_id, enabled)
}

#[tauri::command]
pub fn check_duplicate_request(
    state: tauri::State<'_, Mutex<AppState>>,
    method: String,
    path: String,
    exclude_id: Option<String>,
) -> Result<bool, String> {
    let app_state = lock_or_recover(&state);
    requests::check_duplicate_request(&app_state.db, &method, &path, exclude_id.as_deref())
}
