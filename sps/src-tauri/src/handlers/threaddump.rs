use crate::{
    handlers::types::{ThreadDumpPoint, ThreadDumpThread}, parser::threaddump::Element, store::threaddump::{get_thread_points, get_thread_trace, get_threaddump, get_threaddump_summary}, types::AppState,
};
use std::{ops::Deref, sync::Mutex};
use tauri::command;
use tracing::instrument;

use crate::handlers::types::ThreadDumpSummary;

// Async + spawn_blocking on every query command — see handlers/cpumemstats.rs
// for the rationale.

#[instrument(skip(state))]
#[command]
pub async fn threaddump_dumps(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<ThreadDumpSummary>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        get_threaddump_summary(&cnx)
            .map_err(|e| format!("Error during fetching Thread Dump Summary: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[command]
pub async fn threaddump_threads(
    timestamp: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<ThreadDumpThread>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        get_threaddump(&cnx, timestamp)
            .map_err(|e| format!("Error during fetching Thread Dump: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[command]
pub async fn threaddump_trace(
    tid: u64,
    timestamp: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<Element<'static>>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        get_thread_trace(&cnx, timestamp, tid)
            .map_err(|e| format!("Error during fetching stacktrace from database: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[command]
pub async fn threaddump_thread_series(
    tid: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<ThreadDumpPoint>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    tauri::async_runtime::spawn_blocking(move || {
        get_thread_points(cnx.deref(), tid)
            .map_err(|e| format!("Error during fetching thread points from database: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}
