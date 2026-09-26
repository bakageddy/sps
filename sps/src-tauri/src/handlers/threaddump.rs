use crate::{
    handlers::types::ThreadDumpThread,
    parser::threaddump::Element,
    store::threaddump::{get_thread_trace, get_threaddump, get_threaddump_summary},
    types::AppState,
};
use std::sync::Mutex;
use tauri::command;
use tracing::instrument;

use crate::handlers::types::ThreadDumpSummary;

#[instrument(skip(state))]
#[command]
pub fn threaddump_dumps(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<ThreadDumpSummary>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);
    get_threaddump_summary(&cnx)
        .map_err(|e| format!("Error during fetching Thread Dump Summary: {e}"))
}

#[instrument(skip(state))]
#[command]
pub fn threaddump_threads(
    timestamp: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<ThreadDumpThread>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);
    get_threaddump(&cnx, timestamp).map_err(|e| format!("Error during fetching Thread Dump: {e}"))
}

#[instrument(skip(state))]
#[command]
pub fn threaddump_trace(
    tid: u64,
    timestamp: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<Element<'static>>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);
    get_thread_trace(&cnx, timestamp, tid)
        .map_err(|e| format!("Error during fetching stacktrace from database: {e}"))
}
