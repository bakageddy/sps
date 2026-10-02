use crate::handlers::types::IngestEvent;
use crate::types::AppState;
use crate::util;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::AppHandle;
use tauri::Emitter;
use tracing::instrument;

#[instrument(skip(state))]
#[tauri::command]
pub async fn parse_logs(
    path: String,
    state: tauri::State<'_, Mutex<AppState>>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let store = state.lock().unwrap().store.clone();
    tauri::async_runtime::spawn_blocking(move || {
        app_handle
            .emit(
                "ingest:started",
                IngestEvent::Start {
                    path: PathBuf::from(&path),
                },
            )
            .unwrap();
        if let Err(e) = util::parse_and_persist(path.clone(), store, Some(&app_handle)) {
            return Err(format!("Error during parsing {path}: {e}"));
        };
        app_handle.emit("ingest:finished", ()).unwrap();
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}
