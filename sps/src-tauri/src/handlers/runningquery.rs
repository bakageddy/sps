use crate::{
    handlers::types::{
        BlockingSnapshot, MSSQLLongRunningQuery, MSSQLLongRunningTxn, MSSQLSnapshot,
        PGSQLLongRunningQuery, PGSQLSnapshot, SPWho2Snapshot,
    },
    parser::query::{BlockingQuery, MSSQLQuery, PGSQLQuery, SPWho2Query},
    store::{self, tables::Tables},
    types::AppState,
};
use std::sync::Mutex;
use tauri::State;
use tracing::instrument;

#[instrument(skip(state))]
#[tauri::command]
pub async fn runningquery_pgsql_snapshots(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<PGSQLSnapshot>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    tauri::async_runtime::spawn_blocking(move || {
        store::query::get_pgsql_snapshots(&cnx, Tables::RunningQueryPGSQL)
            .map_err(|e| format!("Error during fetching pgsql running query snapshots: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn runningquery_mssql_snapshots(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<MSSQLSnapshot>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::query::get_mssql_snapshots(&cnx, Tables::RunningQueryMSSQL)
            .map_err(|e| format!("Error during fetching MSSQL snapshots from database: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn runningquery_mssql_blocking_snapshots(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<BlockingSnapshot>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::query::get_mssql_blocking_snapshots(&cnx, Tables::RunningQueryBlockingMSSQL).map_err(
            |e| format!("Error during fetching MSSQL Blocking snapshots from database: {e}"),
        )
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn runningquery_spwho2_snapshots(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<SPWho2Snapshot>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;
    tauri::async_runtime::spawn_blocking(move || {
        store::query::get_mssql_spwho2_snapshots(&cnx).map_err(|e| {
            format!("Error during fetching MSSQL Blocking snapshots from database: {e}")
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn runningquery_pgsql_queries(
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
        store::query::get_pgsql_queries(&cnx, Tables::RunningQueryPGSQL, timestamp)
            .map_err(|e| format!("Error during fetching PGSQL queries from database: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn runningquery_mssql_queries(
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
        store::query::get_mssql_queries(&cnx, Tables::RunningQueryMSSQL, timestamp)
            .map_err(|e| format!("Error during fetching MSSQL queries from database: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn runningquery_spwho2(
    timestamp: u64,
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<SPWho2Query<'static>>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::query::get_mssql_spwho2(&cnx, timestamp)
            .map_err(|e| format!("Error during fetching MSSQL queries from database: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn runningquery_mssql_blocking(
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
        store::query::get_mssql_blocking(&cnx, Tables::RunningQueryBlockingMSSQL, timestamp)
            .map_err(|e| format!("Error during fetching MSSQL queries from database: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn runningquery_mssql_longrunning(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<MSSQLLongRunningQuery>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::query::get_mssql_long_running(&cnx, Tables::RunningQueryMSSQL).map_err(|e| {
            format!("Error during fetching MSSQL Long running queries from database: {e}")
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn runningquery_pgsql_longrunning(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<PGSQLLongRunningQuery>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::query::get_pgsql_long_running(&cnx, Tables::RunningQueryPGSQL).map_err(|e| {
            format!("Error during fetching PGSQL long running queries from database: {e}")
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn runningquery_mssql_longtxns(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<MSSQLLongRunningTxn>, String> {
    let cnx = state
        .lock()
        .unwrap()
        .store
        .get()
        .map_err(|e| format!("Error during obtaining database connection: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
        store::query::get_mssql_long_running_txn(&cnx, Tables::RunningQueryMSSQL)
            .map_err(|e| format!("Error during fetching MSSQL long running txn from database: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}
