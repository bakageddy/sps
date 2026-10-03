use std::sync::Mutex;

use tauri::{State, async_runtime::spawn_blocking};

use crate::store::note;
use crate::{handlers::types::Note, types::AppState};

#[tauri::command]
pub async fn notes_list(state: State<'_, Mutex<AppState>>) -> Result<Vec<Note>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Failed to get connection from the database pool: {}", e))?;
    spawn_blocking(move || note::get_notes(&cnx).map_err(|e| format!("Failed to fetch notes: {e}")))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn notes_upsert(note: Note, state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Failed to get connection from the database pool: {}", e))?;
    spawn_blocking(move || {
        note::upsert(&cnx, note).map_err(|e| format!("Error during updating/insering note: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn notes_delete(
    created_at: u64,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Failed to get connection from the database pool: {}", e))?;

    spawn_blocking(move || {
        note::delete(&cnx, created_at).map_err(|e| format!("Error during delete: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}
