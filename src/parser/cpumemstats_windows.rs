use crate::{
    error::cpumemstats::Parse,
    parser::scanner::Scanner,
    util::{SchemaMapper, ToUnixMillis},
};
use time::{
    Date, PrimitiveDateTime, Time, format_description::FormatItem, macros::format_description,
};
use tracing::warn;

static CPUMEMORYSTATS_TIME_FORMAT: &[FormatItem<'static>] =
    format_description!("[hour]:[minute]:[second].[subsecond]");

static CPUMEMORYSTATS_DATE_FORMAT: &[FormatItem<'static>] =
    format_description!("[day]-[month]-[year]");

#[derive(Debug, PartialEq)]
pub struct CPUStat<'a> {
    pub pid: u64,
    pub cpu_usage: f32,
    pub path: &'a str,
    pub name: &'a str,
}

#[derive(Debug, PartialEq)]
pub struct MemoryStat<'a> {
    pub pid: u64,
    pub memory_usage_mb: f32,
    pub path: &'a str,
    pub name: &'a str,
}

#[derive(Debug, PartialEq)]
pub struct CPUStatTable<'a> {
    pub timestamp_ms: u64,
    pub total_cpu: f32,
    pub stats: Vec<CPUStat<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct MemoryStatTable<'a> {
    pub timestamp_ms: u64,
    pub total_memory: f32,
    pub stats: Vec<MemoryStat<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum CPUMemoryStats<'a> {
    CPU(CPUStatTable<'a>),
    Memory(MemoryStatTable<'a>),
}

impl<'a> TryFrom<&'a str> for CPUStat<'a> {
    type Error = Parse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        scanner.skip_whitespace();
        let name = scanner
            .take_within_exclusive("|", "|")
            .map(|name| name.trim())
            .map_err(Parse::NameExtraction)?;
        let pid = scanner
            .take_within_exclusive("|", "|")
            .map(|pid| pid.trim())
            .map_err(Parse::PIDExtraction)?
            .parse::<u64>()
            .map_err(Parse::PIDParse)?;
        let cpu_usage = scanner
            .take_within_exclusive("|", "|")
            .map(|cpu_usage| cpu_usage.trim())
            .map_err(Parse::CPUUsageExtraction)?
            .parse::<f32>()
            .map_err(Parse::CPUParse)?;
        let path = scanner
            .take_within_exclusive("|", "|")
            .map(|path| path.trim())
            .map_err(Parse::PathExtraction)?;

        Ok(CPUStat {
            pid,
            cpu_usage,
            path,
            name,
        })
    }
}

impl<'a> TryFrom<&'a str> for MemoryStat<'a> {
    type Error = Parse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        scanner.skip_whitespace();
        let name = scanner
            .take_within_exclusive("|", "|")
            .map(|name| name.trim())
            .map_err(Parse::NameExtraction)?;

        let pid = scanner
            .take_within_exclusive("|", "|")
            .map(|pid| pid.trim())
            .map_err(Parse::PIDExtraction)?
            .parse::<u64>()
            .map_err(Parse::PIDParse)?;

        let memory_usage_mb = scanner
            .take_within_exclusive("|", "|")
            .map(|mem| mem.trim())
            .map_err(Parse::MemoryUsageExtraction)?
            .parse::<f32>()
            .map_err(Parse::MemoryParse)?;

        let path = scanner
            .take_within_exclusive("|", "|")
            .map(|path| path.trim())
            .map_err(Parse::PathExtraction)?;

        Ok(Self {
            pid,
            memory_usage_mb,
            path,
            name,
        })
    }
}

fn extract_timestamp_ms(line: &str) -> Result<u64, Parse> {
    let mut scanner = Scanner::new(line);
    let time = scanner
        .take_within("[", "]")
        .map_err(Parse::TimestampExtraction)?;
    let date = scanner
        .take_within("[", "]")
        .map_err(Parse::TimestampExtraction)?;
    let time = Time::parse(time, CPUMEMORYSTATS_TIME_FORMAT)?;
    let date = Date::parse(date, CPUMEMORYSTATS_DATE_FORMAT)?;
    let datetime = PrimitiveDateTime::new(date, time);
    Ok(datetime.to_unix_millis().unwrap_or(0))
}

fn extract_total_usage(line: &str) -> Result<f32, Parse> {
    let mut scanner = Scanner::new(line);
    let _ = scanner
        .take_within_exclusive("|", "|")
        .map_err(Parse::TotalUsageExtraction)?;
    let usage = scanner
        .take_within_exclusive("|", "|")
        .map_err(Parse::TotalUsageExtraction)?;
    usage
        .trim()
        .parse::<f32>()
        .map_err(Parse::TotalUsageParsing)
}

impl<'a> TryFrom<&'a str> for CPUStatTable<'a> {
    type Error = Parse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        scanner.skip_whitespace();
        let header = scanner
            .take_until("\n")
            .ok_or_else(|| Parse::HeaderExtraction)?;
        let timestamp_ms = extract_timestamp_ms(header)?;
        let _ = scanner.take_until("\n");
        let log_type = scanner
            .take_within("|", "|")
            .map_err(Parse::LogTypeExtraction)?
            .trim();

        if log_type != "CPU Log" {
            return Err(Parse::InvalidLogType);
        }

        for _ in 0..4 {
            let _ = scanner.take_until("\n");
            scanner.skip_whitespace();
        }

        let mut total_cpu = 0.0f32;
        let mut stats: Vec<CPUStat> = Vec::with_capacity(30);
        while let Some(line) = scanner.take_until("\n") {
            if !line.starts_with("|") {
                break;
            }

            if line.contains("Total CPU") {
                total_cpu = extract_total_usage(line)?;
                break;
            }

            if line.trim_start().starts_with("|-") {
                continue;
            }

            match CPUStat::try_from(line) {
                Ok(stat) => stats.push(stat),
                Err(e) => warn!("Error during parsing CPU stat: {e}"),
            }
        }

        Ok(Self {
            timestamp_ms,
            total_cpu,
            stats,
        })
    }
}

impl<'a> TryFrom<&'a str> for MemoryStatTable<'a> {
    type Error = Parse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        scanner.skip_whitespace();
        let header = scanner
            .take_until("\n")
            .ok_or_else(|| Parse::HeaderExtraction)?;
        let timestamp_ms = extract_timestamp_ms(header)?;
        let _ = scanner.take_until("\n");
        let log_type = scanner
            .take_within("|", "|")
            .map_err(Parse::LogTypeExtraction)?
            .trim();

        if log_type != "Memory Log" {
            return Err(Parse::InvalidLogType);
        }

        for _ in 0..3 {
            let _ = scanner.take_until("\n");
        }

        let mut total_memory = None;
        let mut stats: Vec<MemoryStat> = Vec::with_capacity(30);

        while let Some(line) = scanner.take_until("\n") {
            if !line.starts_with("|") {
                break;
            }

            if line.contains("Total Memory") {
                total_memory = Some(extract_total_usage(line)?);
                break;
            }

            if line.trim().is_empty() {
                break;
            }

            match MemoryStat::try_from(line) {
                Ok(stat) => stats.push(stat),
                Err(e) => warn!("Error during parsing CPU stat {line} : {e}"),
            }
        }

        if total_memory.is_none() {
            warn!("Total Memory Usage not found. Continuing with total memory usage as 0");
        }

        let total_memory = total_memory.unwrap_or(0.0f32);

        Ok(Self {
            timestamp_ms,
            total_memory,
            stats,
        })
    }
}

impl<'a> TryFrom<&'a str> for CPUMemoryStats<'a> {
    type Error = Parse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if value.contains("CPU Log") || value.contains("Total CPU") {
            let table = CPUStatTable::try_from(value)?;
            Ok(CPUMemoryStats::CPU(table))
        } else if value.contains("Memory Log") || value.contains("Total Memory") {
            let table = MemoryStatTable::try_from(value)?;
            Ok(CPUMemoryStats::Memory(table))
        } else {
            Err(Parse::InvalidLogType)
        }
    }
}

impl<'a> SchemaMapper for CPUStat<'a> {
    type Item = (u64, f32, &'a str, &'a str, bool);

    fn map_to_row(&self) -> Self::Item {
        (self.pid, self.cpu_usage, self.path, self.name, true)
    }
}

impl<'a> SchemaMapper for MemoryStat<'a> {
    type Item = (u64, f32, &'a str, &'a str, bool);

    fn map_to_row(&self) -> Self::Item {
        (self.pid, self.memory_usage_mb, self.path, self.name, false)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::cpumemstats_windows::{CPUStat, CPUStatTable, MemoryStat, MemoryStatTable};

    #[test]
    fn cpumemstats_windows_cpu_stat_parse() {
        let input = r#"|  7za.exe              |  18744       |  24             |  C:\Program Files\ManageEngine\ServiceDesk\tools\archiver\windows\x86-64\7za.exe                      |"#;
        let parsed = CPUStat::try_from(input);
        assert!(
            parsed.is_ok(),
            "Error: Failed to parse CPU Stat: {:?}",
            parsed.unwrap_err()
        );

        let parsed = parsed.unwrap();
        assert_eq!(parsed.name, "7za.exe");
        assert_eq!(parsed.pid, 18744);
        assert_eq!(parsed.cpu_usage, 24.0f32);
        assert_eq!(
            parsed.path,
            r#"C:\Program Files\ManageEngine\ServiceDesk\tools\archiver\windows\x86-64\7za.exe"#
        );
    }

    #[test]
    fn cpumemstats_windows_memory_stat_parse() {
        let input = r#"|  java.exe                     |  16084       |  3089.8       |  C:\Program Files\ManageEngine\ServiceDesk\jre\bin\java.exe                                                 |"#;
        let parsed = MemoryStat::try_from(input);
        assert!(
            parsed.is_ok(),
            "Error: Failed to parse Memory Stat: {:?}",
            parsed.unwrap_err()
        );

        let parsed = parsed.unwrap();
        assert_eq!(parsed.name, "java.exe");
        assert_eq!(parsed.pid, 16084);
        assert_eq!(parsed.memory_usage_mb, 3089.8f32);
        assert_eq!(
            parsed.path,
            r#"C:\Program Files\ManageEngine\ServiceDesk\jre\bin\java.exe"#
        );
    }

    #[test]
    fn cpumemstats_windows_cpu_stat_table() {
        let input = r#"
[14:30:24.598]|[11-02-2026]|[CPUMemStatistics]|[INFO]|[156]| :: 
|-----------------------------------------------------------------------------------------------------------------------------------------|
|                    CPU Log                                                                                                              |
|------------------------|--------------|-----------------|-------------------------------------------------------------------------------|
|  Name                  |  Process ID  |  CPU Usage (%)  |  Path                                                                         |
|------------------------|--------------|-----------------|-------------------------------------------------------------------------------|
|  java.exe              |  16084       |  59.14          |  C:\Program Files\ManageEngine\ServiceDesk\jre\bin\java.exe                   |
|  System                |  4           |  3.58           |                                                                               |
|  postgres.exe          |  17404       |  3.17           |  C:\Program Files\ManageEngine\ServiceDesk\pgsql\bin\postgres.exe             |
|  postgres.exe          |  19284       |  2.97           |  C:\Program Files\ManageEngine\ServiceDesk\pgsql\bin\postgres.exe             |
|  postgres.exe          |  14076       |  2.77           |  C:\Program Files\ManageEngine\ServiceDesk\pgsql\bin\postgres.exe             |
|  postgres.exe          |  14648       |  2.6            |  C:\Program Files\ManageEngine\ServiceDesk\pgsql\bin\postgres.exe             |
|  postgres.exe          |  17452       |  2.4            |  C:\Program Files\ManageEngine\ServiceDesk\pgsql\bin\postgres.exe             |
|  postgres.exe          |  1204        |  1.72           |  C:\Program Files\ManageEngine\ServiceDesk\pgsql\bin\postgres.exe             |
|  Taskmgr.exe           |  9488        |  1.42           |  C:\WINDOWS\system32\taskmgr.exe                                              |
|  postgres.exe          |  8960        |  1.05           |  C:\Program Files\ManageEngine\ServiceDesk\pgsql\bin\postgres.exe             |
|  postgres.exe          |  11128       |  1.01           |  C:\Program Files\ManageEngine\ServiceDesk\pgsql\bin\postgres.exe             |
|  svchost.exe           |  4232        |  0.78           |  C:\WINDOWS\system32\svchost.exe                                              |
|  powershell.exe        |  15372       |  0.68           |  C:\WINDOWS\System32\WindowsPowerShell\v1.0\powershell.exe                    |
|  postgres.exe          |  7868        |  0.51           |  C:\Program Files\ManageEngine\ServiceDesk\pgsql\bin\postgres.exe             |
|  postgres.exe          |  9780        |  0.37           |  C:\Program Files\ManageEngine\ServiceDesk\pgsql\bin\postgres.exe             |
|  postgres.exe          |  17884       |  0.34           |  C:\Program Files\ManageEngine\ServiceDesk\pgsql\bin\postgres.exe             |
|  WmiPrvSE.exe          |  11656       |  0.3            |  C:\WINDOWS\system32\wbem\wmiprvse.exe                                        |
|  MonitoringAgent.exe   |  13504       |  0.17           |  C:\Program Files (x86)\Site24x7\WinAgent\monitoring\bin\MonitoringAgent.exe  |
|  explorer.exe          |  10700       |  0.1            |  C:\WINDOWS\Explorer.EXE                                                      |
|  CSFalconService.exe   |  9536        |  0.1            |                                                                               |
|  csrss.exe             |  3232        |  0.1            |                                                                               |
|  WmiPrvSE.exe          |  8128        |  0.1            |  C:\WINDOWS\sysWOW64\wbem\wmiprvse.exe                                        |
|  msedge.exe            |  3876        |  0.07           |  C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe                 |
|  dwm.exe               |  8740        |  0.07           |  C:\WINDOWS\system32\dwm.exe                                                  |
|  svchost.exe           |  1916        |  0.07           |  C:\WINDOWS\System32\svchost.exe                                              |
|  DCProcessMonitor.exe  |  13280       |  0.07           |  C:\Program Files (x86)\ManageEngine\UEMS_Agent\bin\DCProcessMonitor.exe      |
|  WmiPrvSE.exe          |  1900        |  0.07           |  C:\WINDOWS\sysWOW64\wbem\wmiprvse.exe                                        |
|  WmiPrvSE.exe          |  6960        |  0.07           |  C:\WINDOWS\system32\wbem\wmiprvse.exe                                        |
|  postgres.exe          |  10712       |  0.03           |  C:\Program Files\ManageEngine\ServiceDesk\pgsql\bin\postgres.exe             |
|  msedge.exe            |  4640        |  0.03           |  C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe                 |
|  Total CPU             |  86.01       |                 |                                                                               |

        "#;

        let table = CPUStatTable::try_from(input);
        assert!(
            table.is_ok(),
            "Error during parsing {:?}",
            table.unwrap_err()
        );
        let table = table.unwrap();
        let first = table.stats.first();
        assert_eq!(
            first,
            Some(CPUStat {
                name: "java.exe",
                pid: 16084,
                cpu_usage: 59.14f32,
                path: r#"C:\Program Files\ManageEngine\ServiceDesk\jre\bin\java.exe"#
            })
            .as_ref()
        );
        assert_eq!(table.stats.len(), 30);
        assert_eq!(table.total_cpu, 86.01f32);
    }

    #[test]
    fn cpumemstats_windows_memory_stat_table() {
        let input = r#"
[14:30:24.598]|[11-02-2026]|[CPUMemStatistics]|[INFO]|[156]| :: 
|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|                    Memory Log                                                                                                                                               |
|-------------------------------|--------------|---------------|--------------------------------------------------------------------------------------------------------------|
|  Name                         |  Process ID  |  Memory (MB)  |  Path                                                                                                        |
|-------------------------------|--------------|---------------|--------------------------------------------------------------------------------------------------------------|
|  java.exe                     |  16084       |  2693.33      |  C:\Program Files\ManageEngine\ServiceDesk\jre\bin\java.exe                                                  |
|  explorer.exe                 |  10700       |  284.09       |  C:\WINDOWS\Explorer.EXE                                                                                     |
|  CSFalconService.exe          |  9536        |  118.78       |                                                                                                              |
|  MsMpEng.exe                  |  13524       |  104.13       |                                                                                                              |
|  Taskmgr.exe                  |  9488        |  96.2         |  C:\WINDOWS\system32\taskmgr.exe                                                                             |
|  msedge.exe                   |  3876        |  89.98        |  C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe                                                |
|  svchost.exe                  |  1916        |  86.77        |  C:\WINDOWS\System32\svchost.exe                                                                             |
|  WmiPrvSE.exe                 |  5848        |  79.69        |  C:\WINDOWS\system32\wbem\wmiprvse.exe                                                                       |
|  dwm.exe                      |  8740        |  56.13        |  C:\WINDOWS\system32\dwm.exe                                                                                 |
|  msedge.exe                   |  14088       |  51.94        |  C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe                                                |
|  msedgewebview2.exe           |  12932       |  51.68        |  C:\Program Files (x86)\Microsoft\EdgeWebView\Application\143.0.3650.96\msedgewebview2.exe                   |
|  svchost.exe                  |  7772        |  43.48        |  C:\WINDOWS\system32\svchost.exe                                                                             |
|  Site24x7ApplogAgent.exe      |  14560       |  42.27        |  C:\Program Files (x86)\Site24x7\WinAgent\monitoring\bin\Applogbin\Site24x7ApplogAgent.exe                   |
|  WmiPrvSE.exe                 |  1900        |  41.24        |  C:\WINDOWS\sysWOW64\wbem\wmiprvse.exe                                                                       |
|  SearchHost.exe               |  11688       |  32.55        |  C:\WINDOWS\SystemApps\MicrosoftWindows.Client.CBS_cw5n1h2txyewy\SearchHost.exe                              |
|  StartMenuExperienceHost.exe  |  11664       |  32.42        |  C:\WINDOWS\SystemApps\Microsoft.Windows.StartMenuExperienceHost_cw5n1h2txyewy\StartMenuExperienceHost.exe   |
|  msedgewebview2.exe           |  12156       |  31.18        |  C:\Program Files (x86)\Microsoft\EdgeWebView\Application\143.0.3650.96\msedgewebview2.exe                   |
|  WmiPrvSE.exe                 |  8128        |  28.72        |  C:\WINDOWS\sysWOW64\wbem\wmiprvse.exe                                                                       |
|  svchost.exe                  |  4180        |  28.41        |  C:\WINDOWS\System32\svchost.exe                                                                             |
|  msedge.exe                   |  14772       |  27.95        |  C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe                                                |
|  postgres.exe                 |  17884       |  27.63        |  C:\Program Files\ManageEngine\ServiceDesk\pgsql\bin\postgres.exe                                            |
|  MonitoringAgent.exe          |  13504       |  27           |  C:\Program Files (x86)\Site24x7\WinAgent\monitoring\bin\MonitoringAgent.exe                                 |
|  powershell.exe               |  15372       |  25.26        |  C:\WINDOWS\System32\WindowsPowerShell\v1.0\powershell.exe                                                   |
|  ShellExperienceHost.exe      |  10372       |  19.49        |  C:\WINDOWS\SystemApps\ShellExperienceHost_cw5n1h2txyewy\ShellExperienceHost.exe                             |
|  svchost.exe                  |  4232        |  19.16        |  C:\WINDOWS\system32\svchost.exe                                                                             |
|  msedge.exe                   |  5492        |  17.7         |  C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe                                                |
|  dwm.exe                      |  1852        |  14.29        |  C:\WINDOWS\system32\dwm.exe                                                                                 |
|  WindowsTerminal.exe          |  14752       |  13.95        |  C:\Program Files\WindowsApps\Microsoft.WindowsTerminal_1.23.20211.0_x64__8wekyb3d8bbwe\WindowsTerminal.exe  |
|  ECEATelemetry.exe            |  7484        |  13.36        |  C:\Program Files (x86)\ManageEngine\UEMS_Agent\EndpointIntelligence\Root\Public\Bin\ECEATelemetry.exe       |
|  WmiPrvSE.exe                 |  6960        |  13.27        |  C:\WINDOWS\system32\wbem\wmiprvse.exe                                                                       |
|  Total Memory                 |  5035.02     |               |                                                                                                              |

            "#;

        let table = MemoryStatTable::try_from(input);
        assert!(
            table.is_ok(),
            "Error during parsing memory table: {:?}",
            table.unwrap_err()
        );
        let table = table.unwrap();
        assert_eq!(table.stats.len(), 30);
        assert_eq!(table.timestamp_ms > 0, true);
    }
}
