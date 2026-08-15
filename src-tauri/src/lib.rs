mod commands;
mod db;
mod example_gen;
mod models;
mod server;
mod template;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::Manager;

pub fn lock_or_recover<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            log::warn!("Mutex poisoned, recovering. Previous holder panicked.");
            poisoned.into_inner()
        }
    }
}

pub struct AppState {
    pub db: Arc<db::Database>,
    pub server_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
    pub shutdown_tx: Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
    pub server_port: Mutex<u16>,
    /// Temporary requests stored in memory (key: "METHOD:/path"), shared with running server
    pub temp_requests: Arc<Mutex<HashMap<String, models::TempRequest>>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    log::info!("Starting Portfake application");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::collections::get_collections,
            commands::collections::create_collection,
            commands::collections::delete_collection,
            commands::collections::rename_collection,
            commands::collections::set_collection_enabled,
            commands::requests::get_requests,
            commands::requests::create_request,
            commands::requests::delete_request,
            commands::requests::update_request,
            commands::requests::set_request_enabled,
            commands::requests::check_duplicate_request,
            commands::examples::get_examples,
            commands::examples::update_example,
            commands::server::start_server,
            commands::server::stop_server,
            commands::server::get_server_status,
            commands::server::sync_temp_request,
            commands::server::remove_temp_request,
        ])
        .setup(|app: &mut tauri::App| {
            let exe_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| {
                    app.path().executable_dir().unwrap_or_else(|_| {
                        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
                    })
                });

            let db_path = exe_dir.join("portfake.db");
            let db = db::Database::new(&db_path)
                .map_err(|e| format!("Failed to create database: {}", e))?;

            let state = AppState {
                db: Arc::new(db),
                server_handle: Mutex::new(None),
                shutdown_tx: Mutex::new(None),
                server_port: Mutex::new(3210),
                temp_requests: Arc::new(Mutex::new(HashMap::new())),
            };
            app.manage(state);

            log::info!("Portfake setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            log::error!("Error running tauri application: {}", e);
        });
}
