use duckdb::DuckdbConnectionManager;
use memmap2::Advice;
use r2d2::Pool;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use time::PrimitiveDateTime;
use tracing::debug;
use tracing::{Level, span};
use tracing::{info, warn};

use memmap2::Mmap;

use crate::error;
use crate::{
    ingest::{
        cpumemstats_windows::CPUMemStatsIterator, cpumonitoring::CPUMonitoringIterator,
        kind::DBKind, running_queries::RunningQueriesIterator, stuckquery::StuckQueryIterator,
        stuckthread::StuckThreadIterator, threaddump::ThreadDumpIterator,
    },
    parser::{
        cpumemstats_windows::CPUMemoryStats,
        cpumonitoring::CPUThread,
        running_queries::{MSSQLRunningQueryTable, RunningQueryParser, RunningQueryTable},
        stuckquery::{self, StuckQueryTable},
        stuckthread::StuckThread,
        threaddump::ThreadDump,
    },
    persistence::store::Store,
};

pub type Result<T> = std::result::Result<T, crate::error::Error>;

#[derive(Debug, Default)]
pub struct LogFiles {
    pub threaddumps: Vec<PathBuf>,
    pub cpumonitoring: Vec<PathBuf>,
    pub cpumemstats: Vec<PathBuf>,
    pub stuckthreads: Vec<PathBuf>,
    pub stuckqueries: Vec<PathBuf>,
    pub runningqueries: Vec<PathBuf>,
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
                logfiles.threaddumps.push(entry);
            } else if filename.starts_with("stuckqueries") {
                logfiles.stuckqueries.push(entry);
            } else if filename.starts_with("cpumemstats") {
                logfiles.cpumemstats.push(entry);
            } else if filename.starts_with("runningqueries") {
                logfiles.runningqueries.push(entry);
            }
        }
    }
    logfiles.stuckthreads.sort_by_key(|p| {
        p.file_name()
            .and_then(|f| f.to_str())
            .and_then(|f| f.strip_prefix("stuckthreads"))
            .and_then(|f| f.strip_suffix(".txt"))
            .and_then(|n| n.parse::<u8>().ok())
    });

    logfiles.cpumonitoring.sort_by_key(|p| {
        p.file_name()
            .and_then(|f| f.to_str())
            .and_then(|f| f.strip_prefix("CPUMonitoring"))
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

    logfiles.threaddumps.sort_by_key(|p| {
        p.file_name()
            .and_then(|f| f.to_str())
            .and_then(|f| f.strip_prefix("threaddump"))
            .and_then(|f| f.strip_suffix(".txt"))
            .and_then(|n| n.parse::<u8>().ok())
    });

    logfiles.cpumemstats.sort_by_key(|p| {
        p.file_name()
            .and_then(|f| f.to_str())
            .and_then(|f| f.strip_prefix("cpumemstats"))
            .and_then(|f| f.strip_suffix(".txt"))
            .and_then(|n| n.parse::<u8>().ok())
    });

    logfiles.runningqueries.sort_by_key(|p| {
        p.file_name()
            .and_then(|f| f.to_str())
            .and_then(|f| f.strip_prefix("runningqueries"))
            .and_then(|f| f.strip_suffix(".txt"))
            .and_then(|n| n.parse::<u8>().ok())
    });

    logfiles.threaddumps.reverse();
    logfiles.stuckqueries.reverse();
    logfiles.stuckthreads.reverse();
    logfiles.cpumonitoring.reverse();
    logfiles.cpumemstats.reverse();
    logfiles.runningqueries.reverse();

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

pub trait SchemaMapper {
    type Item;
    fn map_to_row(&self) -> Self::Item;
}

pub fn parse_and_persist_stuckthreads<P>(
    entries: Vec<P>,
    pool: Pool<DuckdbConnectionManager>,
) -> Result<()>
where
    P: AsRef<Path>,
{
    let mut maps = Vec::new();
    let mut events = Vec::with_capacity(4096);
    for entry in &entries {
        let map = map_file(entry)?;
        maps.push(map);
    }

    for (map, entry) in maps.iter().zip(&entries) {
        let entry = entry.as_ref().display();
        info!("Phase: Parse {entry}");
        for chunk in StuckThreadIterator(map) {
            let event = match StuckThread::try_from(chunk) {
                Ok(event) => event,
                Err(e) => {
                    debug!("Chunk: {chunk}");
                    warn!("Error during parsing {entry} : {e:?}");
                    continue;
                }
            };
            events.push(event);
        }
    }
    let cnx = pool.get()?;
    let mut appender = cnx.appender("stuckthread_events")?;
    let mut stacktrace_appender = cnx.appender("stuckthread_stacktraces")?;
    info!("Start Phase: Persist Stuck Threads");
    Store::insert_stuckthreads(&mut appender, &mut stacktrace_appender, events)?;
    info!("Finish Phase: Persist Stuck Threads");

    appender.flush()?;
    stacktrace_appender.flush()?;
    Ok(())
}

pub fn parse_and_persist_stuckqueries<P>(
    entries: Vec<P>,
    pool: Pool<DuckdbConnectionManager>,
) -> Result<()>
where
    P: AsRef<Path>,
{
    let mut maps = Vec::new();
    let mut tables = Vec::new();
    for entry in &entries {
        let map = map_file(entry)?;
        maps.push(map);
    }

    if maps.len() == 0 {
        return Ok(());
    }

    let map = &maps[0];
    let kind = DBKind::detect(&map)?;
    for (map, entry) in maps.iter().zip(&entries) {
        let entry = entry.as_ref().display();
        info!("Phase: Parse {entry}");
        let iter = StuckQueryIterator::new(kind.clone(), map).into_iter();
        for chunk in iter {
            let table = match stuckquery::StuckQueryTable::parse(kind.clone(), chunk) {
                Ok(table) => table,
                Err(e) => {
                    debug!("Error during parsing, chunk: {chunk}");
                    warn!("Error during parsing PGSQL Running Query Table: {e}");
                    continue;
                }
            };
            tables.push(table);
        }
    }

    let cnx = pool.get()?;
    let mut iter = tables.into_iter();
    match kind {
        DBKind::PGSQL => {
            let mut appender = cnx.appender("stuckquery_pgsql")?;
            info!("Start Phase: Persist Stuck Query PGSQL");
            while let Some(StuckQueryTable::PGSQL(table)) = iter.next() {
                let _ = Store::insert_stuckquery_pgsql_table(&mut appender, &table)?;
            }
            info!("Finish Phase: Persist Stuck Query PGSQL");
            appender.flush()?;
        }
        DBKind::MSSQL => {
            let mut appender = cnx.appender("stuckquery_mssql")?;
            info!("Start Phase: Persist Stuck Query MSSQL");
            while let Some(StuckQueryTable::MSSQL(table)) = iter.next() {
                let _ = Store::insert_stuckquery_mssql_table(&mut appender, &table)?;
            }
            info!("Finish Phase: Persist Stuck Query MSSQL");
            appender.flush()?;
        }
    };
    Ok(())
}

pub fn parse_and_persist_running_queries<P>(
    entries: Vec<P>,
    pool: Pool<DuckdbConnectionManager>,
) -> Result<()>
where
    P: AsRef<Path>,
{
    let maps: Result<Vec<Mmap>> = entries.iter().map(map_file).collect();
    let maps = maps?;

    let mut pgsql_tables = Vec::new();
    let mut mssql_tables = Vec::new();
    let mut spwho2_tables = Vec::new();
    for (map, entry) in maps.iter().zip(&entries) {
        let entry = entry.as_ref().display();
        info!("Parsing: {entry}");
        let kind = DBKind::detect(&map)?;
        let peekable = RunningQueriesIterator(map).peekable();
        for table in RunningQueryParser::new(kind, peekable) {
            match table {
                Ok(table) => match table {
                    RunningQueryTable::MSSQL(mssql) => match mssql {
                        MSSQLRunningQueryTable::SPWho2(spwho2) => spwho2_tables.push(spwho2),
                        MSSQLRunningQueryTable::RunningQuery(mssql) => mssql_tables.push(mssql),
                    },
                    RunningQueryTable::PGSQL(pgsql) => pgsql_tables.push(pgsql),
                },
                Err(e) => warn!("Error during parsing running query: {e}"),
            };
        }
    }

    let cnx = pool.get()?;
    let mut pgsql_appender = cnx.appender("running_query_pgsql")?;
    let mut mssql_appender = cnx.appender("running_query_mssql")?;
    let mut spwho2_appender = cnx.appender("running_query_spwho2")?;

    let _ = Store::insert_running_query_pgsql(&mut pgsql_appender, pgsql_tables);
    let _ = Store::insert_running_query_mssql(&mut mssql_appender, mssql_tables);
    let _ = Store::insert_running_query_spwho2(&mut spwho2_appender, spwho2_tables);

    let _ = spwho2_appender.flush();
    let _ = mssql_appender.flush();
    let _ = pgsql_appender.flush();
    
    Ok(())
}

pub fn parse_and_persist_cpumonitoring<P>(
    entries: Vec<P>,
    pool: Pool<DuckdbConnectionManager>,
) -> Result<()>
where
    P: AsRef<Path>,
{
    let mut maps = Vec::new();
    let mut threads = Vec::new();
    for entry in &entries {
        let map = map_file(entry)?;
        maps.push(map);
    }
    for (map, entry) in maps.iter().zip(&entries) {
        let entry = entry.as_ref().display();
        info!("Phase: Parse {entry}");
        for chunk in CPUMonitoringIterator(map) {
            let thread = match CPUThread::try_from(chunk) {
                Ok(thread) => thread,
                Err(e) => {
                    match e {
                        error::cpumonitoring::Parse::MonitoringThreadInfoExtraction(_) => continue,
                        _ => {}
                    };
                    debug!("Error during parsing, chunk: {chunk}");
                    warn!("Error during parsing CPUMonitoring Thread: {e}");
                    continue;
                }
            };
            threads.push(thread);
        }
    }
    let cnx = pool.get()?;
    let mut cpu_appender = cnx.appender("cpumonitoring")?;
    let mut trace_appender = cnx.appender("cpumonitoring_traces")?;
    info!("Start Phase: Persist CPUMonitoring");
    let _ = Store::insert_cpumonitoring_threads(&mut cpu_appender, &mut trace_appender, threads)?;
    info!("Finish Phase: Persist CPUMonitoring");
    cpu_appender.flush()?;
    trace_appender.flush()?;
    Ok(())
}

pub fn parse_and_persist_cpumemstats<P>(
    entries: Vec<P>,
    pool: Pool<DuckdbConnectionManager>,
) -> Result<()>
where
    P: AsRef<Path>,
{
    let mut maps = Vec::new();
    let mut stats = Vec::new();
    for entry in &entries {
        let map = map_file(entry)?;
        maps.push(map);
    }
    for (map, entry) in maps.iter().zip(&entries) {
        let entry = entry.as_ref().display();
        info!("Phase: Parse {entry}");
        for chunk in CPUMemStatsIterator(map) {
            let table = match CPUMemoryStats::try_from(chunk) {
                Ok(table) => table,
                Err(e) => {
                    debug!("Error during parsing, chunk: {chunk}");
                    warn!("Error during parsing CPUMonitoring Thread: {e}");
                    continue;
                }
            };
            stats.push(table);
        }
    }

    let cnx = pool.get()?;
    let mut appender = cnx.appender("cpumemstats")?;
    info!("Start Phase: Persist cpumemstats");
    let _ = Store::insert_cpumemstats(&mut appender, stats)?;
    info!("Finish Phase: Persist cpumemstats");

    appender.flush()?;
    Ok(())
}

pub fn parse_and_persist_threaddump<P>(
    entries: Vec<P>,
    pool: Pool<DuckdbConnectionManager>,
) -> Result<()>
where
    P: AsRef<Path>,
{
    let mut maps = Vec::new();
    let mut dumps = Vec::new();
    for entry in &entries {
        let map = map_file(entry)?;
        maps.push(map);
    }
    for (map, entry) in maps.iter().zip(&entries) {
        let entry = entry.as_ref().display();
        info!("Phase: Parse {entry}");
        for chunk in ThreadDumpIterator(map) {
            let dump = match ThreadDump::try_from(chunk) {
                Ok(dump) => dump,
                Err(e) => {
                    debug!("Error during parsing, chunk: {chunk}");
                    warn!("Error during parsing Thread Dump: {e}");
                    continue;
                }
            };
            dumps.push(dump);
        }
    }
    let cnx = pool.get()?;
    let mut appender = cnx.appender("thread")?;
    let mut stacktrace = cnx.appender("thread_stacktrace")?;
    for dump in dumps {
        Store::insert_threaddump(&mut appender, &mut stacktrace, dump)?;
    }
    appender.flush()?;
    stacktrace.flush()?;
    Ok(())
}
