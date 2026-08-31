use crate::parser::{cpumemstats, cpumonitoring};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("CPUMonitoring: {0}")]
    CPUMonitoring(#[from] cpumonitoring::error::Error),
    #[error("cpumemstats: {0}")]
    CPUMemStats(#[from] cpumemstats::error::Error),
}
