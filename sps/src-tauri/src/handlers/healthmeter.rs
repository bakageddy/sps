use crate::AppState;
use crate::store::healthmeter;
use std::sync::Mutex;
use tauri::State;
use tauri::async_runtime::spawn_blocking;

#[tauri::command]
pub async fn healthmeter_info(state: State<'_, Mutex<AppState>>) -> Result<Option<String>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during fetching database connection: {e}"))?;
    spawn_blocking(move || {
        healthmeter::get_timezone_info(&cnx)
            .map_err(|e| format!("Error during fetching healthmeter info: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}
