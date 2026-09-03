use memmap2::Mmap;

#[cfg(unix)]
use memmap2::Advice;
use std::{
    fs,
    ops::Deref,
    path::{Path, PathBuf},
};
use time::{Date, PlainDateTime, Time, format_description::BorrowedFormatItem};
use tracing::{info, warn};

use crate::{
    parser::{
        cpumemstats::CPUMemStatsParser, cpumonitoring::CPUMonitoringParser,
        stuckthread::StuckthreadParser,
    },
    store::{self, Store},
    types::{LogFiles, ParseInt, TimestampError},
};

pub type Result<T> = std::result::Result<T, crate::error::Error>;

pub fn map_file<P>(path: P) -> std::io::Result<Mmap>
where
    P: AsRef<Path>,
{
    let handle = std::fs::File::open(path)?;
    let map = unsafe { memmap2::Mmap::map(&handle)? };
    if !cfg!(target_os = "windows") {
        let _ = map.advise(Advice::Sequential)?;
    }
    Ok(map)
}

pub fn unix_timestamp_millis(
    time: &str,
    date: &str,
    time_fmt: &[BorrowedFormatItem],
    date_fmt: &[BorrowedFormatItem],
) -> std::result::Result<u64, TimestampError> {
    let parsed_time = Time::parse(time, time_fmt)?;
    let parsed_date = Date::parse(date, date_fmt)?;

    let timestamp = PlainDateTime::new(parsed_date, parsed_time)
        .assume_utc()
        .unix_timestamp_nanos();
    let timestamp = timestamp / 1_000_000;
    let timestamp = u64::try_from(timestamp)?;
    Ok(timestamp)
}

pub fn parse_comma_separated_u64(value: &str) -> std::result::Result<u64, ParseInt> {
    let mut result = 0u64;
    for c in value.bytes() {
        match c {
            b'0'..=b'9' => {
                result *= 10;
                result += (c - b'0') as u64;
            }
            b',' => continue,
            c => return Err(ParseInt::InvalidDigit(c)),
        };
    }
    Ok(result)
}

/// Reads the root directory and strips the entry file with the suffix and prefix
/// and sorts them by reverse, as seen in log files like serverout0..49
/// where 49 is the oldest and must be processed first and 0 is the latest
pub fn get_files_reverse_sort<P>(
    root: P,
    prefix: &str,
    suffix: &str,
) -> std::io::Result<Vec<PathBuf>>
where
    P: AsRef<Path>,
{
    let mut entries: Vec<_> = fs::read_dir(root)?
        .flat_map(|e| e.ok())
        .flat_map(|e| {
            if e.file_name().to_string_lossy().starts_with(prefix) {
                Some(e.path())
            } else {
                None
            }
        })
        .flat_map(|e| e.canonicalize())
        .collect();
    entries.sort_by_key(|p| {
        p.file_name()
            .and_then(|f| f.to_str())
            .and_then(|f| f.strip_suffix(suffix))
            .and_then(|f| f.strip_prefix(prefix))
            .and_then(|f| f.parse::<u8>().ok())
    });

    entries.reverse();

    Ok(entries)
}

pub fn get_files<P>(root: P) -> std::io::Result<LogFiles>
where
    P: AsRef<Path>,
{
    let cpumonitoring = get_files_reverse_sort(&root, "CPUMonitoring", ".txt")?;
    let cpumemstats = get_files_reverse_sort(&root, "cpumemstats", ".txt")?;
    let stuckthreads = get_files_reverse_sort(&root, "stuckthreads", ".txt")?;
    return Ok(LogFiles {
        cpumonitoring,
        cpumemstats,
        stuckthreads,
    });
}

pub fn parse_and_persist<P>(root: P, store: Store) -> Result<()>
where
    P: AsRef<Path>,
{
    let LogFiles {
        cpumonitoring,
        cpumemstats,
        stuckthreads,
    } = get_files(root)?;
    std::thread::scope(|s| {
        let _ = s.spawn(|| -> Result<()> {
            let result = parse_cpumemstats_and_persist(cpumemstats, store.clone());
            if let Err(ref e) = result {
                warn!("Error during parsing/persisting: {e}");
            }
            result
        });
        let _ = s.spawn(|| -> Result<()> {
            let result = parse_cpumonitoring_and_persist(cpumonitoring, store.clone());
            if let Err(ref e) = result {
                warn!("Error during parsing/persisting: {e}");
            }
            result
        });

        let _ = s.spawn(|| -> Result<()> {
            let result = parse_stuckthreads_and_persist(stuckthreads, store.clone());
            if let Err(ref e) = result {
                warn!("Error during parsing/persisting: {e}");
            }
            result
        });
        // TODO:
        // let _cd
        // let _cpumemstats
        // let _threaddump
        // let _stuckthread
        // let _running_queries
        // let _stuck_queries
        // let _query_monitoring
        // let _pgsql_log
        // let _access_log
    });
    Ok(())
}

pub fn parse_cpumonitoring_and_persist(entries: Vec<PathBuf>, store: Store) -> Result<()>
where
{
    let entries = entries
        .into_iter()
        .flat_map(|e| -> Result<(Mmap, PathBuf)> { Ok((map_file(&e)?, e)) });

    let cnx = store.get()?;
    for (mmap, entry) in entries {
        info!("Parsing and Persisting: {:?}", entry.display());
        let result = CPUMonitoringParser::try_from(mmap.deref());
        if let Ok(parser) = result {
            let _ = store::append_cpumonitoring(
                &cnx,
                parser.into_iter().flat_map(|item| {
                    if item.is_ok() {
                        item.ok()
                    } else {
                        warn!(
                            "Error during parsing {:?} due to {}",
                            entry.display(),
                            item.unwrap_err()
                        );
                        None
                    }
                }),
            )?;
        } else {
            warn!(
                "Cannot convert bytes of {:?} to UTF8 due to {:?}",
                entry.display(),
                result.unwrap_err()
            );
            continue;
        }
    }
    Ok(())
}

pub fn parse_cpumemstats_and_persist(entries: Vec<PathBuf>, store: Store) -> Result<()> {
    let entries = entries
        .into_iter()
        .flat_map(|e| -> Result<(Mmap, PathBuf)> { Ok((map_file(&e)?, e)) });

    let cnx = store.get()?;
    for (mmap, entry) in entries {
        info!("Parsing and Persisting: {:?}", entry.display());
        let parser = CPUMemStatsParser::try_from(mmap.deref());
        if let Ok(parser) = parser {
            let _ = store::append_cpumemstats(
                &cnx,
                parser.into_iter().flat_map(|item| {
                    if item.is_ok() {
                        item.ok()
                    } else {
                        warn!(
                            "Error during parsing {:?} due to {}",
                            entry.display(),
                            item.unwrap_err()
                        );
                        None
                    }
                }),
            )?;
        } else {
            warn!(
                "Cannot convert bytes of {:?} to UTF8 due to {:?}",
                entry.display(),
                parser.unwrap_err()
            );
            continue;
        }
    }
    Ok(())
}

pub fn parse_stuckthreads_and_persist(entries: Vec<PathBuf>, store: Store) -> Result<()> {
    let entries = entries
        .into_iter()
        .flat_map(|e| -> Result<(Mmap, PathBuf)> { Ok((map_file(&e)?, e)) });

    let cnx = store.get()?;
    for (mmap, entry) in entries {
        info!("Parsing and persisting: {:?}", entry.display());
        let parser = StuckthreadParser::try_from(mmap.deref());
        if let Ok(parser) = parser {
            let _ = store::append_stuckthread(
                &cnx,
                parser.into_iter().flat_map(|item| {
                    if item.is_ok() {
                        item.ok()
                    } else {
                        warn!(
                            "Error during parsing {:?} due to {}",
                            entry.display(),
                            item.unwrap_err()
                        );
                        None
                    }
                }),
            )?;
        } else {
            warn!(
                "Cannot convert bytes of {:?} to UTF8 due to {:?}",
                entry.display(),
                parser.unwrap_err()
            );
            continue;
        }
    }
    Ok(())
}
