use crate::{
    handlers::types::{CPUMemoryDumpSummary, CPUMemoryPoint, ProcessSeries, ProcessUsage},
    store,
    types::AppState,
};
use std::sync::Mutex;
use tracing::instrument;

// Every query command is async and runs its (blocking, DuckDB has no async
// API) store call on the blocking pool: a slow or blocked query can never
// freeze the webview, and the eight-at-once fetches of the incident page run
// in parallel instead of queuing on the main thread. The Mutex guard is a
// temporary dropped at the end of the `let` — never held across an .await.

#[instrument(skip(state))]
#[tauri::command]
pub async fn cpumem_dumps(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<CPUMemoryDumpSummary>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::cpumemstats::get_cpu_memory_summary(&cnx)
            .map_err(|e| format!("Error during fetching cpumemstats dump summary: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn cpumem_cpu_processes(
    timestamp: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<ProcessUsage>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::cpumemstats::get_cpu_processes(&cnx, timestamp)
            .map_err(|e| format!("Error during fetching cpumemstats dump summary: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn cpumem_mem_processes(
    timestamp: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<ProcessUsage>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::cpumemstats::get_mem_processes(&cnx, timestamp)
            .map_err(|e| format!("Error during fetching cpumemstats dump summary: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn cpumem_series(
    pid: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<ProcessSeries, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::cpumemstats::get_cpumem_series(&cnx, pid)
            .map_err(|e| format!("Error during fetching cpumemstats series: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn cpumem_path_series(
    path: Option<String>,
    name: Option<String>,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<ProcessSeries, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::cpumemstats::get_cpumem_path_series(&cnx, path, name)
            .map_err(|e| format!("Error during fetching cpumemstats series: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn cpumem_total_cpu(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<CPUMemoryPoint>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::cpumemstats::get_cpumem_cpu_total_series(&cnx)
            .map_err(|e| format!("Error during fetching cpumemstats series: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn cpumem_total_memory(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<CPUMemoryPoint>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::cpumemstats::get_cpumem_mem_total_series(&cnx)
            .map_err(|e| format!("Error during fetching cpumemstats series: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}
