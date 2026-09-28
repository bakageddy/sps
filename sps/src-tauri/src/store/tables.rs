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
    RunningQueryPGSQL,
    RunningQueryMSSQL,
    RunningQueryBlockingMSSQL,
    RunningQuerySPWho2,
    ConnectionDump,
    ConnectionDumpTraces,
    Threaddump,
    ThreaddumpThreads,
    ThreaddumpTraces,
}

impl Tables {
    pub fn as_str(&self) -> &'static str {
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
            Self::Threaddump => "threaddump",
            Self::ThreaddumpTraces => "threaddump_traces",
            Self::ThreaddumpThreads => "threaddump_threads",
            Self::RunningQueryPGSQL => "runningquery_pgsql",
            Self::RunningQueryMSSQL => "runningquery_mssql",
            Self::RunningQueryBlockingMSSQL => "runningquery_mssql_blocking",
            Self::RunningQuerySPWho2 => "runningquery_mssql_spwho2",
        }
    }
}

impl Display for Tables {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
