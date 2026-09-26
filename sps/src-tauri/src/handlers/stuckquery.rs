use std::sync::Mutex;

use tauri::State;
use tracing::instrument;

use crate::handlers::types::{
    BlockingSnapshot, MSSQLLongRunningQuery, MSSQLLongRunningTxn, MSSQLSnapshot,
    PGSQLLongRunningQuery, PGSQLSnapshot,
};
use crate::parser::stuckquery::{BlockingQuery, PGSQLQuery, RunningQuery};
use crate::store;
use crate::types::AppState;

// Async + spawn_blocking on every query command — see handlers/cpumemstats.rs
// for the rationale. The row types that borrow (`PGSQLQuery<'a>` etc.) come
// back from the store fully owned (Cow::Owned), so the commands return the
// `'static` instantiation — an unconstrained lifetime on an async command is
// exactly what the command macro can't express.

#[instrument(skip(state))]
#[tauri::command]
pub async fn stuckquery_mssql_snapshots(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<MSSQLSnapshot>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::stuckquery::get_stuckquery_mssql_snapshots(&cnx)
            .map_err(|e| format!("Error during fetching MSSQL snapshots from database: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn stuckquery_mssql_blocking_snapshots(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<BlockingSnapshot>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::stuckquery::get_stuckquery_mssql_blocking_snapshots(&cnx).map_err(|e| {
            format!("Error during fetching MSSQL Blocking snapshots from database: {e}")
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn stuckquery_pgsql_snapshots(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<PGSQLSnapshot>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::stuckquery::get_stuckquery_pgsql_snapshots(&cnx)
            .map_err(|e| format!("Error during fetching PGSQL snapshots from database: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn stuckquery_pgsql_queries(
    timestamp: u64,
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<PGSQLQuery<'static>>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::stuckquery::get_stuckquery_pgsql_queries(&cnx, timestamp)
            .map_err(|e| format!("Error during fetching PGSQL queries from database: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn stuckquery_mssql_queries(
    timestamp: u64,
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<RunningQuery<'static>>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::stuckquery::get_stuckquery_mssql_queries(&cnx, timestamp)
            .map_err(|e| format!("Error during fetching MSSQL queries from database: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn stuckquery_mssql_blocking(
    timestamp: u64,
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<BlockingQuery<'static>>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::stuckquery::get_stuckquery_mssql_blocking(&cnx, timestamp)
            .map_err(|e| format!("Error during fetching MSSQL queries from database: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn stuckquery_mssql_longrunning(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<MSSQLLongRunningQuery>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::stuckquery::get_stuckquery_mssql_long_running(&cnx).map_err(|e| {
            format!("Error during fetching MSSQL Long running queries from database: {e}")
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn stuckquery_pgsql_longrunning(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<PGSQLLongRunningQuery>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::stuckquery::get_stuckquery_pgsql_long_running(&cnx).map_err(|e| {
            format!("Error during fetching PGSQL long running queries from database: {e}")
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn stuckquery_mssql_longtxns(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<MSSQLLongRunningTxn>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::stuckquery::get_stuckquery_mssql_long_running_txn(&cnx)
            .map_err(|e| format!("Error during fetching MSSQL long running txn from database: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}
