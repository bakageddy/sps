use std::sync::Mutex;

use tauri::State;
use tracing::instrument;

use crate::handlers::types::{
    BlockingSnapshot, MSSQLLongRunningQuery, MSSQLLongRunningTxn, MSSQLSnapshot,
    PGSQLLongRunningQuery, PGSQLSnapshot,
};
use crate::parser::query::{BlockingQuery, MSSQLQuery, PGSQLQuery};
use crate::store;
use crate::store::tables::Tables;
use crate::types::AppState;

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
        store::query::get_mssql_snapshots(&cnx, Tables::StuckqueryMSSQL)
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
        store::query::get_mssql_blocking_snapshots(&cnx, Tables::StuckqueryBlockingMSSQL).map_err(
            |e| format!("Error during fetching MSSQL Blocking snapshots from database: {e}"),
        )
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
        store::query::get_pgsql_snapshots(&cnx, Tables::StuckqueryPGSQL)
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
        store::query::get_pgsql_queries(&cnx, Tables::StuckqueryPGSQL, timestamp)
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
) -> Result<Vec<MSSQLQuery<'static>>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::query::get_mssql_queries(&cnx, Tables::StuckqueryMSSQL, timestamp)
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
        store::query::get_mssql_blocking(&cnx, Tables::StuckqueryBlockingMSSQL, timestamp)
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
        store::query::get_mssql_long_running(&cnx, Tables::StuckqueryMSSQL).map_err(|e| {
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
        store::query::get_pgsql_long_running(&cnx, Tables::StuckqueryPGSQL).map_err(|e| {
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
        store::query::get_mssql_long_running_txn(&cnx, Tables::StuckqueryMSSQL)
            .map_err(|e| format!("Error during fetching MSSQL long running txn from database: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}
