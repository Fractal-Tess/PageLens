// State management for the Tauri application.

use pagelens_db::Database;
use tauri::{AppHandle, Manager};

/// Initialize the database and register it as managed state.
/// This should be called during app setup.
pub fn init_database(app: &AppHandle) -> Result<Database, Box<dyn std::error::Error>> {
    let app_local_data_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app local data dir: {}", e))?;

    let db_path = app_local_data_dir.join("pagelens.db");
    let db = Database::open(&db_path)?;

    Ok(db)
}
