use crate::store::Store;
use std::{num::TryFromIntError, path::PathBuf};

pub struct AppState {
    pub store: Store,
}

pub struct LogFiles {
    pub cpumonitoring: Vec<PathBuf>,
    pub cpumemstats: Vec<PathBuf>,
}

#[derive(Debug, thiserror::Error)]
pub enum TimestampError {
    #[error("Time/Date Parsing failed: {0}")]
    Parse(#[from] time::error::Parse),
    #[error("Conversion to u64 failed: {0}")]
    Conversion(#[from] TryFromIntError),
}

#[derive(Debug, thiserror::Error)]
pub enum ParseInt {
    #[error("Invalid Digit found during parsing: {0}")]
    InvalidDigit(u8),
}
