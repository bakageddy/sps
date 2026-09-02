use std::{ops::Deref, sync::Mutex};

use crate::{handlers::types::AggregatedStuckthreadMinimal, store, types::AppState};

#[tauri::command]
pub fn stuckthread_bars(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<AggregatedStuckthreadMinimal>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);

    let result = store::stuckthread::get_stuckthreads_aggregate_minimal(cnx.deref())
        .map_err(|e| format!("Error during fetching stuckthread aggregates: {e}"));

    result
}
