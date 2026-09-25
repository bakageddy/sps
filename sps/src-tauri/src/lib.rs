pub mod arg;
pub mod error;
pub mod handlers;
pub mod parser;
pub mod store;
pub mod types;
pub mod util;

use std::path::PathBuf;
use std::sync::Mutex;

use crate::{arg::Command, store::Store, types::AppState};
use arg::AppArgs;
use clap::Parser;
use handlers::{
    connectiondump::*, cpumemstats::*, cpumonitoring::*, database::*, parse::*, stuckquery::*,
    stuckthread::*,
};
use tauri::Manager;
use tracing::{level_filters::LevelFilter, warn};

pub fn launch(database: Option<PathBuf>) {
    let database = database.clone();
    tauri::Builder::default()
        .setup(|app| {
            let store = Store::init(database)?;
            let state = AppState { store };
            app.manage(Mutex::new(state));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_database,
            database_info,
            parse_logs,
            cpu_dumps,
            cpu_dump_threads,
            cpu_stacktrace,
            cpu_series,
            cpumem_dumps,
            cpumem_cpu_processes,
            cpumem_mem_processes,
            cpumem_series,
            cpumem_path_series,
            cpumem_total_cpu,
            cpumem_total_memory,
            stuckthread_listview,
            stuckthread_trace,
            stuckquery_mssql_blocking_snapshots,
            stuckquery_mssql_snapshots,
            stuckquery_pgsql_snapshots,
            stuckquery_mssql_queries,
            stuckquery_pgsql_queries,
            stuckquery_mssql_blocking,
            stuckquery_pgsql_longrunning,
            stuckquery_mssql_longrunning,
            stuckquery_mssql_longtxns,
            connectiondump_signals,
            connectiondump_pool_stats,
            connectiondump_snapshots,
            connectiondump_traces,
            connectiondump_holders,
        ])
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn run() {
    let subscriber = tracing_subscriber::fmt()
        .with_file(true)
        .with_thread_ids(false)
        .with_max_level(LevelFilter::DEBUG)
        .log_internal_errors(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Unable to init logging system");
    let args = AppArgs::parse();
    if let Some(command) = args.command {
        match command {
            Command::Launch { database } => {
                launch(database);
            }
            Command::Parse { path, database, .. } => {
                let store = match Store::init(database) {
                    Ok(x) => x,
                    Err(e) => {
                        warn!("Cannot initialize database due to: {e}");
                        std::process::exit(1);
                    }
                };

                if let Err(e) = util::parse_and_persist(&path, store) {
                    warn!(
                        "Error during parsing/persisting entries from {:?}: {e}",
                        path.display()
                    )
                };
            }
        }
    } else {
        launch(None);
    }
}
