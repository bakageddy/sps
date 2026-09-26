use tracing::instrument;

use crate::{
    AppState,
    handlers::types::{ConnectionDumpHolder, ConnectionDumpSnapshot},
    parser::connectiondump::{Signal, Stats, Trace},
    store,
};
use std::sync::Mutex;

// Async + spawn_blocking on every query command — see handlers/cpumemstats.rs
// for the rationale; the incident page fires most of these at once.

#[instrument(skip(state))]
#[tauri::command]
pub async fn connectiondump_signals(
    from: Option<u64>,
    to: Option<u64>,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<Signal>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::connectiondump::get_connectiondump_signals(&cnx, from, to)
            .map_err(|e| format!("Error during fetching ConnectionDump signals: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn connectiondump_pool_stats(
    from: Option<u64>,
    to: Option<u64>,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<Stats>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtain database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::connectiondump::get_connectiondump_stats(&cnx, from, to)
            .map_err(|e| format!("Error during fetching ConnectionDump stats: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn connectiondump_snapshots(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<ConnectionDumpSnapshot>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::connectiondump::get_connectiondump_snapshots(&cnx)
            .map_err(|e| format!("Error during fetching connection dump snapshots {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn connectiondump_traces(
    timestamp: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<Trace<'static>>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::connectiondump::get_connectiondump_traces(&cnx, timestamp)
            .map_err(|e| format!("Error during fetching connection dump traces: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn connectiondump_holders(
    from: Option<u64>,
    to: Option<u64>,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<ConnectionDumpHolder>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::connectiondump::get_connectiondump_holders(&cnx, from, to)
            .map_err(|e| format!("Error during fetching connection dump holders: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn connectiondump_threaddump(
    timestamp: u64,
    tolerance: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Option<u64>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::connectiondump::get_connectiondump_threaddump(&cnx, timestamp, tolerance)
            .map_err(|e| format!("Error during resolving the incident's thread dump: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn connectiondump_cpumonitoring(
    timestamp: u64,
    tolerance: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Option<u64>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::connectiondump::get_connectiondump_cpumonitoring(&cnx, timestamp, tolerance)
            .map_err(|e| format!("Error during resolving the incident's CPU monitoring dump: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn connectiondump_cpumemstats(
    timestamp: u64,
    tolerance: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Option<u64>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::connectiondump::get_connectiondump_cpumemstats(&cnx, timestamp, tolerance)
            .map_err(|e| format!("Error during resolving the incident's CPU/Mem stats dump: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}
