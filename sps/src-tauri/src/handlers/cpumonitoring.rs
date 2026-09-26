use crate::{
    AppState,
    handlers::types::{CPUPoint, CPUThread, DumpSummary},
    store::{self, types::Frame},
};
use std::sync::Mutex;
use tracing::instrument;

// Async + spawn_blocking on every query command — see handlers/cpumemstats.rs
// for the rationale; same pattern throughout.

#[instrument(skip(state))]
#[tauri::command]
pub async fn cpu_stacktrace(
    tid: u64,
    timestamp: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Option<Vec<Frame>>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::cpumonitoring::get_stackframes(&cnx, tid, timestamp)
            .map_err(|e| format!("Error during fetching stack frames from database due to: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn cpu_dumps(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<DumpSummary>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::cpumonitoring::get_cpu_dumps(&cnx)
            .map_err(|e| format!("Error during fetching CPUMonitoring dump summary: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn cpu_dump_threads(
    timestamp: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<CPUThread>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::cpumonitoring::get_cpu_dump_threads(&cnx, timestamp)
            .map_err(|e| format!("Error during fetching CPUMonitoring dump summary: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn cpu_series(
    tid: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<CPUPoint>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::cpumonitoring::get_cpu_series(&cnx, tid)
            .map_err(|e| format!("Error during fetching CPUMonitoring Series: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}
