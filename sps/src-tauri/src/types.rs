use crate::store::Store;
use std::path::PathBuf;

pub struct AppState {
    pub store: Store,
}

pub struct LogFiles {
    pub cpumonitoring: Vec<PathBuf>,
    pub cpumemstats: Vec<PathBuf>,
}
