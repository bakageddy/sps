use crate::handlers::types::{FragFlag, FragPoint, FragSnapshot};
use crate::parser::query::FragmentationRatioEntry;
use crate::store;
use crate::types::AppState;
use std::sync::Mutex;
use tauri::State;
use tauri::async_runtime::spawn_blocking;

#[tauri::command]
pub async fn fragmentation_snapshots(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<FragSnapshot>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Failed to obtain database connection from the pool: {e}"))?;

    spawn_blocking(move || {
        store::fragmentation_ratio::get_fragmentation_snapshots(&cnx).map_err(|e| {
            format!("Failed to get the fragmentation snapshots from the database: {e}")
        })
    })
    .await
    .unwrap()
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fragmentation_indices(
    timestamp: u64,
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<FragmentationRatioEntry<'static>>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Failed to obtain database connection from the pool: {e}"))?;

    spawn_blocking(move || {
        store::fragmentation_ratio::get_fragmentation_indices(&cnx, timestamp)
            .map_err(|e| format!("Failed to get the fragmentation indices from the database: {e}"))
    })
    .await
    .unwrap()
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fragmentation_index_series(
    schema: String,
    table: String,
    index: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<FragPoint>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Failed to obtain database connection from the pool: {e}"))?;

    spawn_blocking(move || {
        store::fragmentation_ratio::get_fragmentation_series(&cnx, schema, table, index).map_err(
            |e| format!("Failed to get the fragmentation ratio series from the database: {e}"),
        )
    })
    .await
    .unwrap()
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fragmentation_flagged(
    min_pages: u64,
    min_ratio: f32,
    state: State<'_, Mutex<AppState>>
) -> Result<Vec<FragFlag>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Failed to obtain database connection from the pool: {e}"))?;

    spawn_blocking(move || {
        store::fragmentation_ratio::get_fragmentation_flagged(&cnx, min_pages, min_ratio)
            .map_err(|e| format!("Failed to get the fragmentation ratio flagged from the database: {e}"))
    })
    .await
    .unwrap()
    .map_err(|e| e.to_string())
}

