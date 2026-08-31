use crate::util;
use crate::{handlers::types::ParseReport, types::AppState};
use std::sync::Mutex;

// TODO: implement parse status
#[tauri::command]
pub fn parse_logs(
    path: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<ParseReport, String> {
    let guard = state.lock().unwrap();
    let store = guard.store.clone();
    drop(guard);

    if let Err(e) = util::parse_and_persist(path.clone(), store) {
        return Err(format!("Error during parsing {path}: {e}"));
    };

    Ok(ParseReport::default())
}
