use std::fmt::Display;

pub enum Tables {
    CPUMonitoring,
    CPUMonitoringStackTraces,
    WindowsCPUStats,
    WindowsMemoryStats,
    LinuxStats,
    Stuckthread,
    StuckthreadTraces,
    StuckqueryPGSQL,
    StuckqueryMSSQL,
    StuckqueryBlockingMSSQL,
    ConnectionDump,
    ConnectionDumpTraces,
}

impl Tables {
    pub fn into_str(&self) -> &'static str {
        match self {
            Self::CPUMonitoringStackTraces => "cpumonitoring_stacktraces",
            Self::CPUMonitoring => "cpumonitoring",
            Self::WindowsCPUStats => "windows_cpu_stats",
            Self::WindowsMemoryStats => "windows_memory_stats",
            Self::LinuxStats => "linux_stats",
            Self::Stuckthread => "stuckthread",
            Self::StuckthreadTraces => "stuckthread_traces",
            Self::StuckqueryPGSQL => "stuckquery_pgsql",
            Self::StuckqueryMSSQL => "stuckquery_mssql",
            Self::StuckqueryBlockingMSSQL => "stuckquery_mssql_blocking",
            Self::ConnectionDump => "connectiondump",
            Self::ConnectionDumpTraces => "connectiondump_stacktraces",
        }
    }
}

impl Display for Tables {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.into_str())
    }
}
