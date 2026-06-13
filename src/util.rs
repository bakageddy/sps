use memmap2::Advice;
use std::fs;
use std::io;
use std::iter::Sum;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use time::PrimitiveDateTime;

use memmap2::Mmap;

pub type Result<T> = std::result::Result<T, crate::error::Error>;

#[derive(Debug, Default)]
pub struct LogFiles {
    pub threaddump: Vec<PathBuf>,
    pub cpumonitoring: Vec<PathBuf>,
    pub stuckthreads: Vec<PathBuf>,
    pub stuckqueries: Vec<PathBuf>,
}

pub fn get_logfiles_sorted(entries: impl Iterator<Item = PathBuf>) -> LogFiles {
    let mut logfiles: LogFiles = Default::default();
    for entry in entries {
        if let Some(filename) = entry.file_name().and_then(|e| e.to_str()) {
            if filename.starts_with("CPUMonitoring") {
                logfiles.cpumonitoring.push(entry);
            } else if filename.starts_with("stuckthread") {
                logfiles.stuckthreads.push(entry);
            } else if filename.starts_with("threaddump") {
                logfiles.threaddump.push(entry);
            } else if filename.starts_with("stuckqueries") {
                logfiles.stuckqueries.push(entry);
            }
        }
    }
    logfiles.stuckthreads.sort_by_key(|p| {
        p.file_name()
            .and_then(|f| f.to_str())
            .and_then(|f| f.strip_prefix("stuckthread"))
            .and_then(|f| f.strip_suffix(".txt"))
            .and_then(|n| n.parse::<u8>().ok())
    });

    logfiles.cpumonitoring.sort_by_key(|p| {
        p.file_name()
            .and_then(|f| f.to_str())
            .and_then(|f| f.strip_prefix("cpumonitoring"))
            .and_then(|f| f.strip_suffix(".txt"))
            .and_then(|n| n.parse::<u8>().ok())
    });

    logfiles.stuckqueries.sort_by_key(|p| {
        p.file_name()
            .and_then(|f| f.to_str())
            .and_then(|f| f.strip_prefix("stuckqueries"))
            .and_then(|f| f.strip_suffix(".txt"))
            .and_then(|n| n.parse::<u8>().ok())
    });

    logfiles.threaddump.sort_by_key(|p| {
        p.file_name()
            .and_then(|f| f.to_str())
            .and_then(|f| f.strip_prefix("threaddump"))
            .and_then(|f| f.strip_suffix(".txt"))
            .and_then(|n| n.parse::<u8>().ok())
    });

    logfiles
}

pub fn get_entries<P>(root: P) -> io::Result<impl Iterator<Item = PathBuf>>
where
    P: AsRef<Path>,
{
    Ok(fs::read_dir(root)?.flat_map(|e| e.ok()).map(|e| e.path()))
}

pub fn map_file<P>(path: P) -> self::Result<Mmap>
where
    P: AsRef<Path>,
{
    let handle = fs::File::open(path)?;
    let map = unsafe { memmap2::Mmap::map(&handle)? };
    let _ = map.advise(Advice::WillNeed)?;
    Ok(map)
}

pub fn parse_num<T>(value: &str) -> std::result::Result<T, <T as FromStr>::Err>
where
    T: FromStr + Sum + Default + PartialOrd,
{
    value.parse::<T>()
}

pub fn parse_comma_separated_u32(value: &str) -> Option<u32> {
    let mut result = 0u32;
    for c in value.bytes() {
        match c {
            b'0'..=b'9' => {
                result *= 10;
                result += (c - b'0') as u32;
            }
            b',' => continue,
            _ => return None,
        };
    }
    Some(result)
}

pub trait ToUnixMillis {
    fn to_unix_millis(&self) -> Option<u64>;
}

impl ToUnixMillis for PrimitiveDateTime {
    fn to_unix_millis(&self) -> Option<u64> {
        let result = self.assume_utc().unix_timestamp_nanos() / 1_000_000;
        u64::try_from(result).ok()
    }
}
