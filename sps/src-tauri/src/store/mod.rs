pub mod cpumemstats;
pub mod cpumonitoring;
pub mod error;
pub mod stuckthread;
pub mod tables;
pub mod types;

use std::{iter, path::Path};

use duckdb::{Connection, DuckdbConnectionManager, params};
use r2d2::{Pool, PooledConnection};

use crate::{
    parser::{
        cpumemstats::StatTable,
        cpumonitoring::CPUMonitoring,
        stuckquery::{MSSQLQuery, Stuckquery, StuckqueryTable},
        stuckthread::Stuckthread,
    },
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
                    traces_appender.append_row((start, tid, idx, frame.method, frame.source))?;
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

pub fn append_stuckqueries<'a>(
    cnx: &Connection,
    iter: impl Iterator<Item = StuckqueryTable<'a>>,
) -> Result<(), store::error::Error> {
    let mut pgsql_appender = cnx.appender_to_db(Tables::StuckqueryPGSQL.into_str(), "main")?;
    let mut mssql_appender = cnx.appender_to_db(Tables::StuckqueryMSSQL.into_str(), "main")?;
    let mut block_appender =
        cnx.appender_to_db(Tables::StuckqueryBlockingMSSQL.into_str(), "main")?;
    cnx.appender_to_db(Tables::StuckqueryBlockingMSSQL.into_str(), "main")?;
    for result in iter {
        for query in result.queries {
            match query {
                Stuckquery::PGSQL(pgsql) => {
                    pgsql_appender.append_row((
                        result.timestamp,
                        pgsql.pid,
                        pgsql.query_time,
                        pgsql.txn_time,
                        pgsql.db_name,
                        pgsql.state.into(),
                        pgsql.waiting,
                        pgsql.query,
                        pgsql.state_change,
                        pgsql.application_name,
                        pgsql.client_addr,
                        pgsql.client_host,
                        pgsql.client_port,
                    ))?;
                }
                Stuckquery::MSSQL(mssql) => match mssql {
                    MSSQLQuery::Running(mssql) => {
                        mssql_appender.append_row(params![
                            result.timestamp,
                            mssql.session_id,
                            mssql.status.into_str(),
                            mssql.txn_id,
                            mssql.blocked_by,
                            mssql.wait_type.map(|w| w.as_str()),
                            mssql.wait_resource,
                            mssql.wait_time_ms,
                            mssql.cpu_time_ms,
                            mssql.logical_reads,
                            mssql.reads,
                            mssql.writes,
                            mssql.elapsed,
                            mssql.statement,
                            mssql.command_text,
                            mssql.command,
                            mssql.login,
                            mssql.host,
                            mssql.db,
                            mssql.program,
                            mssql.host_process,
                            mssql.last_request_end,
                            mssql.login_time,
                            mssql.open_txn
                        ])?;
                    }
                    MSSQLQuery::Blocking(query) => {
                        block_appender.append_row((
                            result.timestamp,
                            query.head_blocker,
                            query.session_id,
                            query.txn_id,
                            query.blocking_session_id,
                            query.wait_type.map(|w| w.as_str()),
                            query.wait_duration,
                            query.wait_resource,
                            query.statement_start_offset,
                            query.statement_end_offset,
                            query.plan_handle,
                            query.sql_handle,
                            query.most_recent_sql_handle,
                            query.level,
                            query.blocker_query_or_most_recent_query,
                        ))?;
                    }
                },
            }
        }
    }
    Ok(())
}
