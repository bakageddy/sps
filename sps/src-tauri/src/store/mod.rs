pub mod cpumemstats;
pub mod cpumonitoring;
pub mod error;
pub mod stuckthread;
pub mod tables;
pub mod types;

use std::{iter, path::Path};

use duckdb::{Connection, DuckdbConnectionManager};
use r2d2::{Pool, PooledConnection};

use crate::{
    parser::{cpumemstats::StatTable, cpumonitoring::CPUMonitoring, stuckthread::Stuckthread},
    store::{self, tables::Tables},
};

pub struct Store(Pool<DuckdbConnectionManager>);

impl Store {
    pub fn pool(&self) -> Pool<DuckdbConnectionManager> {
        self.0.clone()
    }
    pub fn path(&self) -> Result<Option<String>, store::error::Error> {
        let cnx = self.0.get()?;
        let path = cnx.path().map(|p| p.to_string_lossy().to_string());
        Ok(path)
    }
    pub fn init<P>(path: Option<P>) -> Result<Self, store::error::Error>
    where
        P: AsRef<Path>,
    {
        let mgr = if let Some(path) = path {
            DuckdbConnectionManager::file(path)?
        } else {
            DuckdbConnectionManager::memory()?
        };

        let schema = include_str!("../../schema.sql");
        let pool = Pool::builder().max_size(12).build(mgr)?;
        let cnx = pool.get()?;
        cnx.execute_batch(schema)?;

        Ok(Self(pool))
    }

    pub fn get(&self) -> Result<PooledConnection<DuckdbConnectionManager>, store::error::Error> {
        Ok(self.0.get()?)
    }
}

impl Clone for Store {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub fn append_cpumonitoring<'a>(
    cnx: &Connection,
    iter: impl Iterator<Item = CPUMonitoring<'a>>,
) -> Result<(), store::error::Error> {
    let mut cpu_appender = cnx.appender_to_db(Tables::CPUMonitoring.into_str(), "main")?;
    let mut trace_appender =
        cnx.appender_to_db(Tables::CPUMonitoringStackTraces.into_str(), "main")?;
    for item in iter {
        cpu_appender.append_row((
            item.tid,
            item.timestamp,
            item.usage,
            item.state.into_str(),
            item.name,
        ))?;

        if let Some(frames) = item.trace {
            for (idx, frame) in iter::zip(0.., frames.0) {
                trace_appender.append_row((
                    item.tid,
                    item.timestamp,
                    idx,
                    frame.method,
                    frame.source,
                ))?;
            }
        }
    }

    cpu_appender.flush()?;
    trace_appender.flush()?;
    Ok(())
}

pub fn append_cpumemstats<'a>(
    cnx: &Connection,
    iter: impl Iterator<Item = StatTable<'a>>,
) -> Result<(), store::error::Error> {
    let mut linux_stat = cnx.appender_to_db(Tables::LinuxStats.into_str(), "main")?;
    let mut windows_cpu = cnx.appender_to_db(Tables::WindowsCPUStats.into_str(), "main")?;
    let mut windows_mem = cnx.appender_to_db(Tables::WindowsMemoryStats.into_str(), "main")?;

    for table in iter {
        match table {
            StatTable::WCPU(cputable) => {
                windows_cpu.append_rows(cputable.stats.iter().map(|s| {
                    (
                        cputable.timestamp,
                        cputable.total,
                        &s.path,
                        s.cpu,
                        s.pid,
                        &s.name,
                    )
                }))?;
            }
            StatTable::WMEM(memtable) => {
                windows_mem.append_rows(memtable.stats.iter().map(|s| {
                    (
                        memtable.timestamp,
                        memtable.total,
                        &s.path,
                        s.mem,
                        s.pid,
                        &s.name,
                    )
                }))?;
            }
            StatTable::UNIX(unix) => linux_stat.append_rows(unix.stats.iter().map(|s| {
                (
                    unix.timestamp,
                    unix.cpu,
                    unix.mem,
                    &s.user,
                    &s.name,
                    s.pid,
                    s.cpu,
                    s.mem,
                    &s.path,
                )
            }))?,
        }
    }

    linux_stat.flush()?;
    windows_cpu.flush()?;
    windows_mem.flush()?;
    Ok(())
}

pub fn append_stuckthread<'a>(
    cnx: &Connection,
    iter: impl Iterator<Item = Stuckthread<'a>>,
) -> Result<(), store::error::Error> {
    let mut appender = cnx.appender_to_db(Tables::Stuckthread.into_str(), "main")?;
    let mut traces_appender = cnx.appender_to_db(Tables::StuckthreadTraces.into_str(), "main")?;
    for event in iter {
        let (timestamp, tid, duration, name, request, active) = match event {
            Stuckthread::Begin {
                start,
                tid,
                duration,
                name,
                request,
                trace,
                active,
            } => {
                for (idx, frame) in (0..).zip(trace.0) {
                    traces_appender.append_row((
                        start,
                        tid,
                        idx,
                        frame.method,
                        frame.source,
                    ))?;
                }
                (start, tid, duration, name, Some(request), active)
            }
            Stuckthread::End {
                end: timestamp,
                tid,
                duration,
                name,
                active,
            } => (timestamp, tid, duration, name, None, active),
        };
        appender.append_row((timestamp, tid, duration, name, request, active))?;
    }
    Ok(())
}
