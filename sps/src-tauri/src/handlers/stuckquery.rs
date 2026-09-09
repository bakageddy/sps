use std::{ops::Deref, sync::Mutex};

use tauri::State;

use crate::handlers::types::{BlockingSnapshot, MSSQLSnapshot, PGSQLSnapshot};
use crate::parser::stuckquery::PGSQLQuery;
use crate::store;
use crate::types::AppState;

#[tauri::command]
pub fn stuckquery_mssql_snapshots(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<MSSQLSnapshot>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);

    let result = store::stuckquery::get_stuckquery_mssql_snapshots(cnx.deref())
        .map_err(|e| format!("Error during fetching MSSQL snapshots from database: {e}"));
    result
}

#[tauri::command]
pub fn stuckquery_mssql_blocking_snapshots(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<BlockingSnapshot>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);

    let result = store::stuckquery::get_stuckquery_mssql_blocking_snapshots(cnx.deref())
        .map_err(|e| format!("Error during fetching MSSQL Blocking snapshots from database: {e}"));
    result
}

#[tauri::command]
pub fn stuckquery_pgsql_snapshots(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<PGSQLSnapshot>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);

    let result = store::stuckquery::get_stuckquery_pgsql_snapshots(cnx.deref())
        .map_err(|e| format!("Error during fetching PGSQL snapshots from database: {e}"));
    result
}

#[tauri::command]
pub fn stuckquery_pgsql_queries<'a>(
    timestamp: u64,
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<PGSQLQuery<'a>>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);
    let result = store::stuckquery::get_stuckquery_pgsql_queries(cnx.deref(), timestamp)
        .map_err(|e| format!("Error during fetching PGSQL queries from database: {e}"));
    result
}
