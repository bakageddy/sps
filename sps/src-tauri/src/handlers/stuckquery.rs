use std::{ops::Deref, sync::Mutex};

use tauri::State;

use crate::handlers::types::{
    BlockingSnapshot, MSSQLLongRunningQuery, MSSQLLongRunningTxn, MSSQLSnapshot,
    PGSQLLongRunningQuery, PGSQLSnapshot,
};
use crate::parser::stuckquery::{BlockingQuery, PGSQLQuery, RunningQuery};
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

#[tauri::command]
pub fn stuckquery_mssql_queries<'a>(
    timestamp: u64,
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<RunningQuery<'a>>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);

    let result = store::stuckquery::get_stuckquery_mssql_queries(cnx.deref(), timestamp)
        .map_err(|e| format!("Error during fetching MSSQL queries from database: {e}"));

    result
}

#[tauri::command]
pub fn stuckquery_mssql_blocking<'a>(
    timestamp: u64,
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<BlockingQuery<'a>>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);
    let result = store::stuckquery::get_stuckquery_mssql_blocking(cnx.deref(), timestamp)
        .map_err(|e| format!("Error during fetching MSSQL queries from database: {e}"));
    result
}

#[tauri::command]
pub fn stuckquery_mssql_longrunning(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<MSSQLLongRunningQuery>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);
    let result = store::stuckquery::get_stuckquery_mssql_long_running(cnx.deref()).map_err(|e| {
        format!("Error during fetching MSSQL Long running queries from database: {e}")
    });
    result
}

#[tauri::command]
pub fn stuckquery_pgsql_longrunning(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<PGSQLLongRunningQuery>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);

    let result = store::stuckquery::get_stuckquery_pgsql_long_running(cnx.deref()).map_err(|e| {
        format!("Error during fetching PGSQL long running queries from database: {e}")
    });
    result
}

#[tauri::command]
pub fn stuckquery_mssql_longtxns(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<MSSQLLongRunningTxn>, String> {
    let guard = state.lock().unwrap();
    let cnx = guard
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    drop(guard);

    let result = store::stuckquery::get_stuckquery_mssql_long_running_txn(cnx.deref())
        .map_err(|e| format!("Error during fetching MSSQL long running txn from database: {e}"));
    result
}
