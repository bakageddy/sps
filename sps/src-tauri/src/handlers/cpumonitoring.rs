use crate::{
    AppState,
    handlers::types::{CPUPoint, CPUThread, DumpSummary},
    store::{self, types::Frame},
};
use std::{ops::Deref, sync::Mutex};

#[tauri::command]
pub fn cpu_stacktrace(
    tid: u64,
    timestamp: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Option<Vec<Frame>>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);

    let result = store::cpumonitoring::get_stackframes(cnx.deref(), tid, timestamp)
        .map_err(|e| format!("Error during fetching stack frames from database due to: {e}"));

    result
}

#[tauri::command]
pub fn cpu_dumps(state: tauri::State<'_, Mutex<AppState>>) -> Result<Vec<DumpSummary>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);

    let result = store::cpumonitoring::get_cpu_dumps(cnx.deref())
        .map_err(|e| format!("Error during fetching CPUMonitoring dump summary: {e}"));
    result
}

#[tauri::command]
pub fn cpu_dump_threads(
    timestamp: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<CPUThread>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);

    let result = store::cpumonitoring::get_cpu_dump_threads(cnx.deref(), timestamp)
        .map_err(|e| format!("Error during fetching CPUMonitoring dump summary: {e}"));
    result
}

#[tauri::command]
pub fn cpu_series(
    tid: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<CPUPoint>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);

    let result = store::cpumonitoring::get_cpu_series(cnx.deref(), tid)
        .map_err(|e| format!("Error during fetching CPUMonitoring Series: {e}"));
    result
}
