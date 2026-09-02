pub enum Tables {
    CPUMonitoring,
    CPUMonitoringStackTraces,
    WindowsCPUStats,
    WindowsMemoryStats,
    LinuxStats,
    Stuckthread,
    StuckthreadTraces,
}

impl Tables {
    pub fn into_str(self) -> &'static str {
        match self {
            Self::CPUMonitoringStackTraces => "cpumonitoring_stacktraces",
            Self::CPUMonitoring => "cpumonitoring",
            Self::WindowsCPUStats => "windows_cpu_stats",
            Self::WindowsMemoryStats => "windows_memory_stats",
            Self::LinuxStats => "linux_stats",
            Self::Stuckthread => "stuckthread",
            Self::StuckthreadTraces => "stuckthread_traces",
        }
    }
}
