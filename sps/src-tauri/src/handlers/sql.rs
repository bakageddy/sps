use std::sync::Mutex;

use tauri::State;
use tauri::async_runtime::spawn_blocking;

use crate::handlers::types::SQLResult;
use crate::store::sql;
use crate::{handlers::types::SQLTable, types::AppState};

#[tauri::command]
pub async fn sql_schema(state: State<'_, Mutex<AppState>>) -> Result<Vec<SQLTable>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining connection from pool: {e}"))?;
    spawn_blocking(move || {
        sql::get_schema(&cnx).map_err(|e| format!("Error during fetching the schema: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn sql_query(
    state: State<'_, Mutex<AppState>>,
    sql: String,
    limit: u64,
    offset: u64,
) -> Result<Option<SQLResult>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining connection from pool: {e}"))?;
    spawn_blocking(move || {
        sql::execute_query(&cnx, &sql, limit, offset)
            .map_err(|e| format!("Failed to execute query: {sql} due to {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn sql_export_csv(
    state: State<'_, Mutex<AppState>>,
    sql: String,
    path: String,
) -> Result<usize, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining connection from pool: {e}"))?;
    spawn_blocking(move || {
        sql::export(&cnx, &sql, &path)
            .map_err(|e| format!("Failed to export/execute query: {sql} to {path} due to {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}
