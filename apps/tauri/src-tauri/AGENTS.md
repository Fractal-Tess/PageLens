# src-tauri Knowledge Base

**Path:** `src-tauri/`  
**Stack:** Tauri v2, Rust, SQLite

## Structure

```
src-tauri/
├── src/
│   ├── main.rs           # Binary entry
│   ├── lib.rs            # App initialization
│   ├── ipc.rs           # IPC command handlers
│   ├── state.rs         # App state (DB)
│   └── commands/        # Command modules
├── icons/               # App icons
├── tauri.conf.json      # Tauri config
└── Cargo.toml           # Crate manifest
```

## IPC Commands

Add commands in `ipc.rs` with Specta:

```rust
#[tauri::command]
#[specta::specta]
fn my_command(arg: String) -> Result<String, Error> {
    Ok(arg)
}
```

Register in `lib.rs`: `.commands(collect_commands![...])`

## ⚠️ Anti-Patterns

| Issue                            | Location   | Fix                                 |
| -------------------------------- | ---------- | ----------------------------------- |
| `clippy::needless_pass_by_value` | `ipc.rs:1` | Use references                      |
| `.unwrap()` on event emit        | `ipc.rs`   | Use `ok()` or proper error handling |

## Build

```bash
bun tauri dev      # Development
bun build          # Production build
```
