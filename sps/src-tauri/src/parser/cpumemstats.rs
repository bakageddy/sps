use serde::Deserialize;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashSet;
use std::str::Utf8Error;
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;

use crate::parser::cpumemstats::error::Error;
use crate::parser::tokenizer::Parser;
use crate::parser::tokenizer::Tokenizer;
use crate::util;

const CPUMEMSTATS_TIME_FORMAT: &[BorrowedFormatItem] =
    format_description!("[hour]:[minute]:[second].[subsecond]");
const CPUMEMSTATS_DATE_FORMAT: &[BorrowedFormatItem] = format_description!("[day]-[month]-[year]");

#[derive(Debug)]
pub struct CPUMemStatsParser<'a>(&'a str, ParserState);

#[derive(Debug)]
enum ParserState {
    Initial,
    Timestamp,

    WindowsCPUHeader,
    WindowsCPUStat,
    WindowsCPUTotal,

    WindowsMemoryHeader,
    WindowsMemoryStat,
    WindowsMemoryTotal,

    UNIXHeader,
    UNIXStat,
    UNIXTotal,
}

impl<'a> CPUMemStatsParser<'a> {
    pub fn new(data: &'a str) -> Self {
        Self(data, ParserState::Initial)
    }
}

impl<'a> TryFrom<&'a [u8]> for CPUMemStatsParser<'a> {
    type Error = Utf8Error;
    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let value = std::str::from_utf8(value)?;
        Ok(Self::new(value))
    }
}

// FIXME: when we return Some(Err(e)), call tok.remaining()
impl<'a> Iterator for CPUMemStatsParser<'a> {
    type Item = Result<StatTable<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut tok = Tokenizer::new(self.0);
        tok.skip_whitespace();
        if tok.is_empty() {
            return None;
        }

        let mut timestamp = None;
        self.1 = ParserState::Initial;

        while let Some(line) = tok.get_line() {
            if line.starts_with("[") {
                if let Some(next) = tok.peek_line()
                    && next.trim_start().starts_with("|")
                {
                    let mut htok = Tokenizer::new(line);
                    let time = match htok.take_within("[", "]").map_err(Error::InvalidFormat) {
                        Ok(time) => time,
                        Err(e) => return Some(Err(e)),
                    };

                    let date = match htok.take_within("[", "]").map_err(Error::InvalidFormat) {
                        Ok(date) => date,
                        Err(e) => return Some(Err(e)),
                    };

                    timestamp = util::unix_timestamp_millis(
                        time,
                        date,
                        CPUMEMSTATS_TIME_FORMAT,
                        CPUMEMSTATS_DATE_FORMAT,
                    )
                    .ok();
                    self.1 = ParserState::Timestamp;
                    break;
                }
            }
        }

        if !matches!(self.1, ParserState::Timestamp) {
            return None;
        }

        let timestamp = timestamp.unwrap_or(0);

        for _ in 0..3 {
            let _ = tok.get_line()?;
        }

        let line = tok.get_line()?;
        let columns = Self::columns(line);
        if columns.contains("User") && columns.contains("PID") {
            self.1 = ParserState::UNIXHeader;
        } else if columns.contains("CPU Usage (%)") && columns.contains("Process ID") {
            self.1 = ParserState::WindowsCPUHeader;
        } else if columns.contains("Memory (MB)") && columns.contains("Process ID") {
            self.1 = ParserState::WindowsMemoryHeader;
        } else {
            return Some(Err(Error::UnableToDetectTableType(String::from(line))));
        }

        let _ = tok.get_line()?;

        match self.1 {
            ParserState::UNIXHeader => {
                self.1 = ParserState::UNIXStat;
                let mut stats = Vec::new();
                while let Some(line) = tok.peek_line()
                    && line.starts_with("|")
                {
                    if line.contains(LinuxStat::PREAMBLE) {
                        self.1 = ParserState::UNIXTotal;
                        break;
                    }

                    let stat = LinuxStat::parse(line);
                    match stat {
                        Ok(stat) => {
                            stats.push(stat);
                        }
                        Err(e) => return Some(Err(e)),
                    }

                    let _ = tok.get_line();
                }

                // BUG: if the current line does not start with |
                // we will stop parsing.
                // This is kinda dumb. Fix it
                if !matches!(self.1, ParserState::UNIXTotal) {
                    return None;
                }

                let total_usage = match LinuxStat::parse_total(tok.get_line()?) {
                    Ok(x) => x,
                    Err(e) => return Some(Err(e)),
                };

                let unix = LinuxStatTable {
                    stats,
                    timestamp,
                    cpu: total_usage.0,
                    mem: total_usage.1,
                };

                self.0 = tok.remaining();

                return Some(Ok(StatTable::UNIX(unix)));
            }
            ParserState::WindowsCPUHeader => {
                self.1 = ParserState::WindowsCPUStat;
                let mut stats = Vec::new();
                while let Some(line) = tok.peek_line()
                    && line.trim().starts_with("|")
                {
                    if line.contains(WindowsCPUStat::PREAMBLE) {
                        self.1 = ParserState::WindowsCPUTotal;
                        break;
                    }

                    match WindowsCPUStat::parse(line) {
                        Ok(x) => stats.push(x),
                        Err(e) => return Some(Err(e)),
                    };

                    let _ = tok.get_line();
                }

                // BUG: if the current line does not start with |
                // we will stop parsing.
                // This is kinda dumb. Fix it
                if !matches!(self.1, ParserState::WindowsCPUTotal) {
                    return None;
                }

                let total = match WindowsCPUStat::parse_total(tok.get_line()?) {
                    Ok(x) => x,
                    Err(e) => return Some(Err(e)),
                };

                self.0 = tok.remaining();

                return Some(Ok(StatTable::WCPU(WindowsCPUTable {
                    stats,
                    total,
                    timestamp,
                })));
            }
            ParserState::WindowsMemoryHeader => {
                self.1 = ParserState::WindowsMemoryStat;
                let mut stats = Vec::new();
                while let Some(line) = tok.peek_line()
                    && line.trim().starts_with("|")
                {
                    if line.contains(WindowsMemoryStat::PREAMBLE) {
                        self.1 = ParserState::WindowsMemoryTotal;
                        break;
                    }

                    match WindowsMemoryStat::parse(line) {
                        Ok(x) => stats.push(x),
                        Err(e) => return Some(Err(e)),
                    };

                    let _ = tok.get_line();
                }

                // BUG: if the current line does not start with | and total is not found for some
                // reason we will stop parsing.
                // This is kinda dumb. Fix it
                if !matches!(self.1, ParserState::WindowsMemoryTotal) {
                    return None;
                }

                let total = match WindowsMemoryStat::parse_total(tok.get_line()?) {
                    Ok(x) => x,
                    Err(e) => return Some(Err(e)),
                };

                self.0 = tok.remaining();

                return Some(Ok(StatTable::WMEM(WindowsMemoryTable {
                    stats,
                    total,
                    timestamp,
                })));
            }
            _ => unreachable!(),
        }
    }
}

impl CPUMemStatsParser<'_> {
    pub fn columns(header: &str) -> HashSet<&str> {
        let mut columns = HashSet::new();
        let mut htok = Tokenizer::new(header);
        while let Ok(inner) = htok.take_within_exclusive("|", "|") {
            columns.insert(inner.trim());
        }
        columns
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum StatTable<'a> {
    UNIX(LinuxStatTable<'a>),
    WCPU(WindowsCPUTable<'a>),
    WMEM(WindowsMemoryTable<'a>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowsMemoryTable<'a> {
    pub stats: Vec<WindowsMemoryStat<'a>>,
    pub total: f32,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowsMemoryStat<'a> {
    pub path: Option<Cow<'a, str>>,
    pub mem: f32,
    pub pid: u64,
    pub name: Cow<'a, str>,
}

impl<'a> Parser<'a> for WindowsMemoryStat<'a> {
    type Error = Error;

    fn parse(data: &'a str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut tok = Tokenizer::new(data);
        let name = tok.take_within_exclusive("|", "|")?.trim().into();
        let pid = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        let mem = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        let path: Cow<'_, str> = tok.take_within_exclusive("|", "|")?.trim().into();
        let path = if path.is_empty() { None } else { Some(path) };

        Ok(Self {
            name,
            pid,
            mem,
            path,
        })
    }
}

impl WindowsMemoryStat<'_> {
    const PREAMBLE: &'static str = "Total Memory";

    pub fn parse_total(data: &str) -> Result<f32, Error> {
        let mut tok = Tokenizer::new(data);
        let preamble = tok.take_within_exclusive("|", "|")?.trim();
        if preamble != Self::PREAMBLE {
            return Err(Error::InvalidPreamble(
                String::from(Self::PREAMBLE),
                String::from(preamble),
            ));
        }
        let mem = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        Ok(mem)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowsCPUTable<'a> {
    pub stats: Vec<WindowsCPUStat<'a>>,
    pub timestamp: u64,
    pub total: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowsCPUStat<'a> {
    pub path: Option<Cow<'a, str>>,
    pub cpu: f32,
    pub pid: u64,
    pub name: Cow<'a, str>,
}

impl<'a> Parser<'a> for WindowsCPUStat<'a> {
    type Error = Error;

    fn parse(data: &'a str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut tok = Tokenizer::new(data);
        let name = tok.take_within_exclusive("|", "|")?.trim().into();
        let pid = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        let cpu = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        let path: Cow<'_, str> = tok.take_within_exclusive("|", "|")?.trim().into();
        let path = if path.is_empty() { None } else { Some(path) };

        Ok(Self {
            name,
            pid,
            cpu,
            path,
        })
    }
}

impl WindowsCPUStat<'_> {
    const PREAMBLE: &'static str = "Total CPU";

    pub fn parse_total(data: &str) -> Result<f32, Error> {
        let mut tok = Tokenizer::new(data);
        let preamble = tok.take_within_exclusive("|", "|")?.trim();
        if preamble != Self::PREAMBLE {
            return Err(Error::InvalidPreamble(
                String::from(Self::PREAMBLE),
                String::from(preamble),
            ));
        }
        let cpu = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        Ok(cpu)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinuxStatTable<'a> {
    pub stats: Vec<LinuxStat<'a>>,
    pub timestamp: u64,
    pub cpu: f32,
    pub mem: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinuxStat<'a> {
    pub user: Cow<'a, str>,
    pub name: Cow<'a, str>,
    pub pid: u64,
    pub cpu: f32,
    pub mem: f32,
    pub path: Cow<'a, str>,
}

impl<'a> Parser<'a> for LinuxStat<'a> {
    type Error = Error;

    fn parse(data: &'a str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut tok = Tokenizer::new(data);
        let user = tok.take_within_exclusive("|", "|")?.trim().into();
        let pid = tok
            .take_within_exclusive("|", "|")?
            .trim()
            .parse()
            .map_err(Error::ParsePID)?;
        let name = tok.take_within_exclusive("|", "|")?.trim().into();
        let cpu = tok
            .take_within_exclusive("|", "|")?
            .trim()
            .parse()
            .map_err(Error::ParseUsage)?;
        let mem = tok
            .take_within_exclusive("|", "|")?
            .trim()
            .parse()
            .map_err(Error::ParseUsage)?;
        let path = tok.take_within_exclusive("|", "|")?.trim().into();

        Ok(Self {
            name,
            user,
            pid,
            cpu,
            mem,
            path,
        })
    }
}

impl LinuxStat<'_> {
    const PREAMBLE: &'static str = "Total CPU Usage";
    pub fn parse_total(data: &str) -> Result<(f32, f32), Error> {
        let mut tok = Tokenizer::new(data);
        let _ = tok.take_within_exclusive("|", "|")?;
        let _ = tok.take_within_exclusive("|", "|")?;
        let preamble = tok.take_within_exclusive("|", "|")?.trim();
        if preamble != Self::PREAMBLE {
            return Err(Error::InvalidPreamble(
                String::from(Self::PREAMBLE),
                String::from(preamble),
            ));
        }

        let cpu = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        let mem = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        Ok((cpu, mem))
    }
}

pub mod error {
    use std::num::{ParseFloatError, ParseIntError};
    #[non_exhaustive]
    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Invalid format for cpu/memory stats table: {0}")]
        InvalidFormat(#[from] crate::parser::tokenizer::error::Error),
        #[error("Parsing/Invalid Format for timestamp: {0}")]
        ParseTimestamp(#[from] time::error::Parse),
        #[error("Parse PID (uint64_t) from string: {0}")]
        ParsePID(#[from] ParseIntError),
        #[error("Parse PID (float32_t) from string: {0}")]
        ParseUsage(#[from] ParseFloatError),
        #[error("Unable to detect table type from header: {0}")]
        UnableToDetectTableType(String),
        #[error("Expected Preamble: {0}, got: {1}")]
        InvalidPreamble(String, String),
    }
}

#[cfg(test)]
mod test {
    use std::ops::Deref;

    use crate::parser::{
        cpumemstats::{
            CPUMemStatsParser, LinuxStat, LinuxStatTable, StatTable, WindowsCPUStat,
            WindowsCPUTable, WindowsMemoryStat, WindowsMemoryTable,
        },
        tokenizer::Parser,
    };
    use crate::util::map_file;

    #[test]
    fn cpumemstats_linux_stat() {
        let line = "|  dinesh-+  |  196716   |  rust-analyzer                                                                          |  0.10625             |  1607.22265625   |  rust-analyzer                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |";
        let stat = LinuxStat::parse(line);
        assert!(
            stat.is_ok(),
            "Error during parsing Linux CPUMemstats: {}",
            stat.unwrap_err()
        );
        let stat = stat.unwrap();
        assert_eq!("dinesh-+", stat.user);
        assert_eq!(196716, stat.pid);
        assert_eq!("rust-analyzer", stat.name);
        assert_eq!(0.10625f32, stat.cpu);
        assert_eq!(1607.22265625f32, stat.mem);
        assert_eq!("rust-analyzer", stat.path);
    }

    #[test]
    fn cpumemstats_windows_cpustat() {
        let line = "|  System               |  4           |  7.33           |                                                                               |";
        let stat = WindowsCPUStat::parse(line);
        assert!(
            stat.is_ok(),
            "Error during parsing Linux CPUMemstats: {}",
            stat.unwrap_err()
        );
        let stat = stat.unwrap();
        assert_eq!("System", stat.name);
        assert_eq!(4, stat.pid);
        assert_eq!(7.33f32, stat.cpu);
        assert_eq!(None, stat.path);
    }

    #[test]
    fn cpumemstats_windows_memstat() {
        let line = "|  MsMpEng.exe                  |  4820        |  84.46        |                                                                                                              |";
        let stat = WindowsMemoryStat::parse(line);
        assert!(
            stat.is_ok(),
            "Error during parsing Linux CPUMemstats: {}",
            stat.unwrap_err()
        );
        let stat = stat.unwrap();
        assert_eq!("MsMpEng.exe", stat.name);
        assert_eq!(4820u64, stat.pid);
        assert_eq!(84.46f32, stat.mem);
        assert_eq!(None, stat.path);
    }

    #[test]
    fn cpumemstats_windows_cputable() {
        let map = map_file("test/cpumemstats/windows_cpu_single_table.txt").unwrap();
        let mut iter = CPUMemStatsParser::try_from(map.deref()).unwrap();
        let table = iter.next();
        assert!(table.is_some());
        let table = table.unwrap();
        assert!(
            table.is_ok(),
            "Error during parsing: {}",
            table.unwrap_err()
        );
        let table = table.unwrap();
        assert!(matches!(table, StatTable::WCPU(_)));

        if let StatTable::WCPU(WindowsCPUTable {
            total,
            timestamp,
            stats,
        }) = table
        {
            assert_eq!(stats.len(), 30);
            assert_eq!(total, 46.86f32);
            assert_ne!(timestamp, 0);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn cpumemstats_linux_table() {
        let map = map_file("test/cpumemstats/linux_single_table.txt").unwrap();
        let mut iter = CPUMemStatsParser::try_from(map.deref()).unwrap();
        let table = iter.next();
        assert!(table.is_some());
        let table = table.unwrap();
        assert!(
            table.is_ok(),
            "Error during parsing: {}",
            table.unwrap_err()
        );
        let table = table.unwrap();
        assert!(matches!(table, StatTable::UNIX(_)));

        if let StatTable::UNIX(LinuxStatTable {
            stats,
            timestamp,
            cpu,
            mem,
        }) = table
        {
            assert_eq!(stats.len(), 30);
            assert_ne!(timestamp, 0);
            assert_eq!(cpu, 163.51249999999985f32);
            assert_eq!(mem, 19160.65234375f32);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn cpumemstats_windows_memtable() {
        let map = map_file("test/cpumemstats/windows_mem_single_table.txt").unwrap();
        let mut iter = CPUMemStatsParser::try_from(map.deref()).unwrap();
        let table = iter.next();
        assert!(table.is_some());
        let table = table.unwrap();
        assert!(
            table.is_ok(),
            "Error during parsing: {}",
            table.unwrap_err()
        );
        let table = table.unwrap();
        assert!(matches!(table, StatTable::WMEM(_)));
        if let StatTable::WMEM(WindowsMemoryTable {
            stats,
            timestamp,
            total,
        }) = table
        {
            assert_eq!(stats.len(), 30);
            assert_ne!(timestamp, 0);
            assert_eq!(total, 15475.55f32);
        } else {
            unreachable!();
        }
    }
}
