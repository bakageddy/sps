use crate::{
    handlers::types::{CPUMemoryDumpSummary, CPUMemoryPoint, ProcessSeries, ProcessUsage},
    store,
    types::AppState,
};
use std::{ops::Deref, sync::Mutex};

#[tauri::command]
pub fn cpumem_dumps(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<CPUMemoryDumpSummary>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);

    let result = store::cpumemstats::get_cpu_memory_summary(cnx.deref())
        .map_err(|e| format!("Error during fetching cpumemstats dump summary: {e}"));

    result
}

#[tauri::command]
pub fn cpumem_cpu_processes(
    timestamp: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<ProcessUsage>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);

    let result = store::cpumemstats::get_cpu_processes(cnx.deref(), timestamp)
        .map_err(|e| format!("Error during fetching cpumemstats dump summary: {e}"));

    result
}

#[tauri::command]
pub fn cpumem_mem_processes(
    timestamp: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<ProcessUsage>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);

    let result = store::cpumemstats::get_mem_processes(cnx.deref(), timestamp)
        .map_err(|e| format!("Error during fetching cpumemstats dump summary: {e}"));

    result
}

#[tauri::command]
pub fn cpumem_series(
    pid: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<ProcessSeries, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining connection: {e}"))?;
    drop(guard);

    let result = store::cpumemstats::get_cpumem_series(cnx.deref(), pid)
        .map_err(|e| format!("Error during fetching cpumemstats series: {e}"));

    result
}

#[tauri::command]
pub fn cpumem_path_series(
    path: Option<String>,
    name: Option<String>,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<ProcessSeries, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining connection: {e}"))?;
    drop(guard);

    let result = store::cpumemstats::get_cpumem_path_series(cnx.deref(), path, name)
        .map_err(|e| format!("Error during fetching cpumemstats series: {e}"));

    result
}

#[tauri::command]
pub fn cpumem_total_cpu(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<CPUMemoryPoint>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining connection: {e}"))?;
    drop(guard);
    let result = store::cpumemstats::get_cpumem_cpu_total_series(cnx.deref())
        .map_err(|e| format!("Error during fetching cpumemstats series: {e}"));

    result
}

#[tauri::command]
pub fn cpumem_total_memory(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<CPUMemoryPoint>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining connection: {e}"))?;
    drop(guard);
    let result = store::cpumemstats::get_cpumem_mem_total_series(cnx.deref())
        .map_err(|e| format!("Error during fetching cpumemstats series: {e}"));

    result
}
