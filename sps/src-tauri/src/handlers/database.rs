use std::sync::Mutex;

use crate::handlers::types::*;
use crate::store::Store;
use crate::types::AppState;

#[tauri::command]
pub fn open_database(
    path: Option<String>,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<DatabaseInfo, String> {
    let store = match Store::init(path.as_ref()) {
        Ok(store) => store,
        Err(e) => return Err(e.to_string()),
    };

    let mut guard = state.lock().unwrap();
    guard.store = store;
    drop(guard);

    Ok(DatabaseInfo { path })
}

#[tauri::command]
pub fn database_info(state: tauri::State<'_, Mutex<AppState>>) -> Result<Option<String>, String> {
    state
        .lock()
        .unwrap()
        .store
        .path()
        .map_err(|e| e.to_string())
}
