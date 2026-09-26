use std::sync::Mutex;

use tracing::instrument;

use crate::{
    handlers::types::AggregatedStuckthread, parser::stuckthread::Frame, store, types::AppState,
};

// Async + spawn_blocking on every query command — see handlers/cpumemstats.rs
// for the rationale. `Frame<'a>` comes back owned, hence `'static`.

#[instrument(skip(state))]
#[tauri::command]
pub async fn stuckthread_listview(
    from: Option<u64>,
    to: Option<u64>,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<AggregatedStuckthread>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::stuckthread::get_stuckthread_aggregates(&cnx, from, to)
            .map_err(|e| format!("Error during fetching stuckthread aggregates: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn stuckthread_trace(
    tid: u64,
    timestamp: u64,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<Frame<'static>>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::stuckthread::get_stuckthread_trace(&cnx, tid, timestamp)
            .map_err(|e| format!("Error during fetching stuckthread stacktrace: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}
