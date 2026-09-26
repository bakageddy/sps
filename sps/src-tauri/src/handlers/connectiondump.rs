use tracing::instrument;

use crate::{
    AppState,
    handlers::types::{ConnectionDumpHolder, ConnectionDumpSnapshot},
    parser::connectiondump::{Signal, Stats, Trace},
    store,
};
use std::sync::Mutex;

#[instrument(skip(state))]
#[tauri::command]
pub fn connectiondump_signals(
    from: Option<u64>,
    to: Option<u64>,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<Signal>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);

    store::connectiondump::get_connectiondump_signals(&cnx, from, to)
        .map_err(|e| format!("Error during fetching ConnectionDump signals: {e}"))
}

#[instrument(skip(state))]
#[tauri::command]
pub fn connectiondump_pool_stats(
    from: Option<u64>,
    to: Option<u64>,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<Stats>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtain database connection: {e}"))?;
    drop(guard);

    store::connectiondump::get_connectiondump_stats(&cnx, from, to)
        .map_err(|e| format!("Error during fetching ConnectionDump stats: {e}"))
}

#[instrument(skip(state))]
#[tauri::command]
pub fn connectiondump_snapshots(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<ConnectionDumpSnapshot>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);

    store::connectiondump::get_connectiondump_snapshots(&cnx)
        .map_err(|e| format!("Error during fetching connection dump snapshots {e}"))
}

#[instrument(skip(state))]
#[tauri::command]
pub fn connectiondump_traces(
    timestamp: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<Trace<'static>>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);

    store::connectiondump::get_connectiondump_traces(&cnx, timestamp)
        .map_err(|e| format!("Error during fetching connection dump traces: {e}"))
}

#[instrument(skip(state))]
#[tauri::command]
pub fn connectiondump_holders(
    from: Option<u64>,
    to: Option<u64>,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<ConnectionDumpHolder>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);

    store::connectiondump::get_connectiondump_holders(&cnx, from, to)
        .map_err(|e| format!("Error during fetching connection dump traces: {e}"))
}

#[instrument(skip(state))]
#[tauri::command]
pub fn connectiondump_threaddump(
    timestamp: u64,
    tolerance: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Option<u64>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);
    store::connectiondump::get_connectiondump_threaddump(&cnx, timestamp, tolerance).map_err(|e| {
        format!("Error during fetching thread dump from the associated connection dump: {e}")
    })
}

#[instrument(skip(state))]
#[tauri::command]
pub fn connectiondump_cpumonitoring(
    timestamp: u64,
    tolerance: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Option<u64>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);
    store::connectiondump::get_connectiondump_cpumonitoring(&cnx, timestamp, tolerance).map_err(
        |e| format!("Error during fetching thread dump from the associated connection dump: {e}"),
    )
}

#[instrument(skip(state))]
#[tauri::command]
pub fn connectiondump_cpumemstats(
    timestamp: u64,
    tolerance: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Option<u64>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);
    store::connectiondump::get_connectiondump_cpumemstats(&cnx, timestamp, tolerance).map_err(
        |e| format!("Error during fetching thread dump from the associated connection dump: {e}"),
    )
}
