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
use handlers::database::*;
use handlers::parse::*;
use handlers::stuckthread::*;
use handlers::{cpumemstats::*, cpumonitoring::*};
use tauri::Manager;
use tracing::warn;
use tracing_subscriber;

// #[cfg_attr(mobile, tauri::mobile_entry_point)]

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
            stuckthread_bars,
        ])
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn run() {
    tracing_subscriber::fmt().init();
    let args = AppArgs::parse();
    if args.command.is_none() {
        launch(None);
    } else {
        let command = args.command.unwrap();
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
    };
}
