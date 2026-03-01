// State management for the Tauri application.

use std::collections::HashMap;
use std::string::ToString;
use std::sync::Mutex;

use pagelens_db::Database;
use tauri::{AppHandle, Builder, Manager, Wry};

/// Register all managed state with the Tauri builder.
pub fn register_managed_state(builder: Builder<Wry>) -> Builder<Wry> {
    let store = Store::default();

    builder.manage(store)
}

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

/// Simple in-memory key-value store (example state).
#[derive(Default)]
pub struct Store {
    store: Mutex<HashMap<String, String>>,
}

impl Store {
    pub fn add_key_val(&self, key: String, val: String) {
        self.store
            .lock()
            .expect("cannot lock store")
            .insert(key, val);
    }
    pub fn read_key(&self, key: &String) -> Option<String> {
        self.store
            .lock()
            .expect("cannot lock store")
            .get(key)
            .map(ToString::to_string)
    }
}
