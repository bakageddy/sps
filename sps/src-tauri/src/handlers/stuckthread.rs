use std::{ops::Deref, sync::Mutex};

use tracing::instrument;

use crate::{
    handlers::types::AggregatedStuckthread, parser::stuckthread::Frame, store, types::AppState,
};

#[instrument(skip(state))]
#[tauri::command]
pub fn stuckthread_listview(
    from: Option<u64>,
    to: Option<u64>,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<AggregatedStuckthread>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);

    

    store::stuckthread::get_stuckthread_aggregates(cnx.deref(), from, to)
        .map_err(|e| format!("Error during fetching stuckthread aggregates: {e}"))
}

#[instrument(skip(state))]
#[tauri::command]
pub fn stuckthread_trace<'a>(
    tid: u64,
    timestamp: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<Frame<'a>>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);

    
    store::stuckthread::get_stuckthread_trace(cnx.deref(), tid, timestamp)
        .map_err(|e| format!("Error during fetching stuckthread stacktrace: {e}"))
}
