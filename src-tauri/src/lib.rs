mod database;
mod email;
mod models;
mod commands;

use tauri::Manager;
use std::sync::Arc;
use tokio::sync::Mutex;
use database::Database;
use std::path::PathBuf;

pub struct AppState {
    pub db: Arc<Mutex<Database>>,
    pub db_path: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let data_dir = handle
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::temp_dir().join("tauri-email-client"));
            std::fs::create_dir_all(&data_dir)?;
            let db_path: PathBuf = data_dir.join("surreal.db");
            println!("SurrealDB path: {}", db_path.display());

            let db = tauri::async_runtime::block_on(Database::new(&db_path))
                .expect("Failed to initialize database");
            let state = AppState {
                db: Arc::new(Mutex::new(db)),
                db_path: db_path.to_string_lossy().to_string(),
            };
            handle.manage(state);
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::fetch_emails,
            commands::send_email,
            commands::mark_as_read,
            commands::mark_as_unread,
            commands::toggle_star,
            commands::delete_email,
            commands::move_to_trash,
            commands::move_to_folder,
            commands::save_account,
            commands::remove_account,
            commands::get_accounts,
            commands::test_connection,
            commands::sync_emails,
            commands::get_db_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
