use crate::parser::{cpumemstats, cpumonitoring, stuckthread};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("CPUMonitoring: {0}")]
    CPUMonitoring(#[from] cpumonitoring::error::Error),
    #[error("cpumemstats: {0}")]
    CPUMemStats(#[from] cpumemstats::error::Error),
    #[error("stuckthreads: {0}")]
    Stuckthread(#[from] stuckthread::error::Error),
    #[error("stuckquery: {0}")]
    Stuckquery(#[from] stuckquery::error::Error),
    #[error("connection dump: {0}")]
    ConnectionDump(#[from] connectiondump::error::Error),
}
