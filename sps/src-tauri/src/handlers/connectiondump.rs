use crate::{AppState, handlers::types::ConnectionDumpSignal, store};
use std::sync::Mutex;

#[tauri::command]
pub fn connectiondump_signals(
    from: Option<u64>,
    to: Option<u64>,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<ConnectionDumpSignal>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);
    let result = store::connectiondump::get_connectiondump_signals(&cnx, from, to)
        .map_err(|e| format!("Error during fetching ConnectionDump signals: {e}"));
    result
}
