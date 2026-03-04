use std::sync::Mutex;
use tauri::{Builder as TauriBuilder, Manager};

mod ipc;
mod state;

use crate::ipc::{mount_ipc_events, register_ipc_handlers};
use crate::state::init_database;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    // Application
    let app = TauriBuilder::default();

    // Plugins
    let app = app.plugin(tauri_plugin_opener::init());

    // Commands
    let app = register_ipc_handlers(app);

    // Setup - initialize database
    let app = app.setup(|app| {
        mount_ipc_events(app);
        let db = init_database(app.handle())?;
        app.manage(Mutex::new(db));
        Ok(())
    });

    // Run
    app.run(tauri::generate_context!())
        .expect("error while running tauri application");
}
