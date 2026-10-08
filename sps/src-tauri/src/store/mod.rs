pub mod connectiondump;
pub mod cpumemstats;
pub mod cpumonitoring;
pub mod error;
pub mod note;
pub mod query;
pub mod sql;
pub mod stuckthread;
pub mod tables;
pub mod threaddump;
pub mod healthmeter;
pub mod types;

use std::{iter, path::Path};

use duckdb::{Connection, DuckdbConnectionManager, params};
use r2d2::{Pool, PooledConnection};

use crate::{
    parser::{
        connectiondump::{ConnectionDumpEntry, Signal, Stats, TraceDump},
        cpumemstats::StatTable,
        cpumonitoring::CPUMonitoring,
        healthmeter::HCell,
        runningquery::{RunningQuery, RunningQueryTable},
        stuckquery::{StuckQuery, StuckQueryTable},
        stuckthread::Stuckthread,
        threaddump::{
            Element::{self},
            ThreadDump,
        },
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

pub fn flush_results(store: Store) -> Result<(), store::error::Error> {
    let cnx = store.get()?;
    let mut cpumonitoring = cnx.appender_to_db(Tables::CPUMonitoring.as_str(), "main")?;
    let mut cpumonitoring_traces =
        cnx.appender_to_db(Tables::CPUMonitoringStackTraces.as_str(), "main")?;
    let mut linux_stat = cnx.appender_to_db(Tables::LinuxStats.as_str(), "main")?;
    let mut windows_cpu = cnx.appender_to_db(Tables::WindowsCPUStats.as_str(), "main")?;
    let mut windows_mem = cnx.appender_to_db(Tables::WindowsMemoryStats.as_str(), "main")?;
    let mut stuckthread = cnx.appender_to_db(Tables::Stuckthread.as_str(), "main")?;
    let mut stuckthread_traces = cnx.appender_to_db(Tables::StuckthreadTraces.as_str(), "main")?;

    let mut pgsql_appender = cnx.appender_to_db(Tables::StuckqueryPGSQL.as_str(), "main")?;
    let mut mssql_appender = cnx.appender_to_db(Tables::StuckqueryMSSQL.as_str(), "main")?;
    let mut block_appender =
        cnx.appender_to_db(Tables::StuckqueryBlockingMSSQL.as_str(), "main")?;
    let mut cd = cnx.appender_to_db(Tables::ConnectionDump.as_str(), "main")?;
    let mut cd_traces = cnx.appender_to_db(Tables::ConnectionDumpTraces.as_str(), "main")?;
    let mut threaddump = cnx.appender_to_db(Tables::Threaddump.as_str(), "main")?;
    let mut threads = cnx.appender_to_db(Tables::ThreaddumpThreads.as_str(), "main")?;
    let mut thread_traces = cnx.appender_to_db(Tables::ThreaddumpTraces.as_str(), "main")?;
    let mut runningquery_pgsql = cnx.appender_to_db(Tables::RunningQueryPGSQL.as_str(), "main")?;
    let mut runningquery_mssql = cnx.appender_to_db(Tables::RunningQueryMSSQL.as_str(), "main")?;
    let mut runningquery_mssql_blocking =
        cnx.appender_to_db(Tables::RunningQueryBlockingMSSQL.as_str(), "main")?;
    let mut runningquery_spwho2 =
        cnx.appender_to_db(Tables::RunningQuerySPWho2.as_str(), "main")?;
    let mut healthmeter = cnx.appender_to_db(Tables::HealthMeter.as_str(), "main")?;

    cpumonitoring.flush()?;
    cpumonitoring_traces.flush()?;
    linux_stat.flush()?;
    windows_cpu.flush()?;
    windows_mem.flush()?;
    stuckthread.flush()?;
    stuckthread_traces.flush()?;
    pgsql_appender.flush()?;
    mssql_appender.flush()?;
    block_appender.flush()?;
    runningquery_pgsql.flush()?;
    runningquery_mssql.flush()?;
    runningquery_mssql_blocking.flush()?;
    runningquery_spwho2.flush()?;
    cd.flush()?;
    cd_traces.flush()?;
    threaddump.flush()?;
    threads.flush()?;
    thread_traces.flush()?;
    healthmeter.flush()?;
    Ok(())
}

pub fn append_cpumonitoring<'a>(
    cnx: &Connection,
    iter: impl Iterator<Item = CPUMonitoring<'a>>,
) -> Result<(), store::error::Error> {
    let mut cpu_appender = cnx.appender_to_db(Tables::CPUMonitoring.as_str(), "main")?;
    let mut trace_appender =
        cnx.appender_to_db(Tables::CPUMonitoringStackTraces.as_str(), "main")?;
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

    Ok(())
}

pub fn append_cpumemstats<'a>(
    cnx: &Connection,
    iter: impl Iterator<Item = StatTable<'a>>,
) -> Result<(), store::error::Error> {
    let mut linux_stat = cnx.appender_to_db(Tables::LinuxStats.as_str(), "main")?;
    let mut windows_cpu = cnx.appender_to_db(Tables::WindowsCPUStats.as_str(), "main")?;
    let mut windows_mem = cnx.appender_to_db(Tables::WindowsMemoryStats.as_str(), "main")?;

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

    Ok(())
}

pub fn append_stuckthread<'a>(
    cnx: &Connection,
    iter: impl Iterator<Item = Stuckthread<'a>>,
) -> Result<(), store::error::Error> {
    let mut appender = cnx.appender_to_db(Tables::Stuckthread.as_str(), "main")?;
    let mut traces_appender = cnx.appender_to_db(Tables::StuckthreadTraces.as_str(), "main")?;
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
    iter: impl Iterator<Item = StuckQueryTable<'a>>,
) -> Result<(), store::error::Error> {
    let mut pgsql_appender = cnx.appender_to_db(Tables::StuckqueryPGSQL.as_str(), "main")?;
    let mut mssql_appender = cnx.appender_to_db(Tables::StuckqueryMSSQL.as_str(), "main")?;
    let mut block_appender =
        cnx.appender_to_db(Tables::StuckqueryBlockingMSSQL.as_str(), "main")?;
    for result in iter {
        for query in result.queries {
            match query {
                StuckQuery::PGSQL(pgsql) => {
                    pgsql_appender.append_row((
                        result.timestamp,
                        pgsql.pid,
                        pgsql.query_time,
                        pgsql.txn_time,
                        pgsql.db_name,
                        pgsql.state.as_str(),
                        pgsql.waiting,
                        pgsql.query,
                        pgsql.state_change,
                        pgsql.application_name,
                        pgsql.client_addr,
                        pgsql.client_host,
                        pgsql.client_port,
                    ))?;
                }
                StuckQuery::MSSQL(mssql) => {
                    mssql_appender.append_row(params![
                        result.timestamp,
                        mssql.session_id,
                        mssql.status.as_str(),
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
                StuckQuery::Blocking(block) => {
                    block_appender.append_row((
                        result.timestamp,
                        block.head_blocker,
                        block.session_id,
                        block.txn_id,
                        block.blocking_session_id,
                        block.wait_type.map(|w| w.as_str()),
                        block.wait_duration,
                        block.wait_resource,
                        block.statement_start_offset,
                        block.statement_end_offset,
                        block.plan_handle,
                        block.sql_handle,
                        block.most_recent_sql_handle,
                        block.level,
                        block.blocker_query_or_most_recent_query,
                    ))?;
                }
            }
        }
    }
    Ok(())
}

pub fn append_runningqueries<'a>(
    cnx: &Connection,
    iter: impl Iterator<Item = RunningQueryTable<'a>>,
) -> Result<(), store::error::Error> {
    let mut pgsql_appender = cnx.appender_to_db(Tables::RunningQueryPGSQL.as_str(), "main")?;
    let mut mssql_appender = cnx.appender_to_db(Tables::RunningQueryMSSQL.as_str(), "main")?;
    let mut block_appender =
        cnx.appender_to_db(Tables::RunningQueryBlockingMSSQL.as_str(), "main")?;
    let mut spwho2_appender = cnx.appender_to_db(Tables::RunningQuerySPWho2.as_str(), "main")?;
    for result in iter {
        for query in result.queries {
            match query {
                RunningQuery::PGSQL(pgsql) => {
                    pgsql_appender.append_row((
                        result.timestamp,
                        pgsql.pid,
                        pgsql.query_time,
                        pgsql.txn_time,
                        pgsql.db_name,
                        pgsql.state.as_str(),
                        pgsql.waiting,
                        pgsql.query,
                        pgsql.state_change,
                        pgsql.application_name,
                        pgsql.client_addr,
                        pgsql.client_host,
                        pgsql.client_port,
                    ))?;
                }
                RunningQuery::MSSQL(mssql) => {
                    mssql_appender.append_row(params![
                        result.timestamp,
                        mssql.session_id,
                        mssql.status.as_str(),
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
                RunningQuery::Blocking(block) => {
                    block_appender.append_row((
                        result.timestamp,
                        block.head_blocker,
                        block.session_id,
                        block.txn_id,
                        block.blocking_session_id,
                        block.wait_type.map(|w| w.as_str()),
                        block.wait_duration,
                        block.wait_resource,
                        block.statement_start_offset,
                        block.statement_end_offset,
                        block.plan_handle,
                        block.sql_handle,
                        block.most_recent_sql_handle,
                        block.level,
                        block.blocker_query_or_most_recent_query,
                    ))?;
                }
                RunningQuery::SPWho2(spwho2) => {
                    spwho2_appender.append_row((
                        result.timestamp,
                        spwho2.spid,
                        spwho2.status.as_str(),
                        spwho2.login,
                        spwho2.hostname,
                        spwho2.blocked_by,
                        spwho2.dbname,
                        spwho2.command,
                        spwho2.cputime,
                        spwho2.diskio,
                        spwho2.lastbatch,
                        spwho2.program_name,
                        spwho2.request_id,
                    ))?;
                }
            }
        }
    }
    Ok(())
}

pub fn append_connectiondump<'a>(
    cnx: &Connection,
    iter: impl Iterator<Item = ConnectionDumpEntry<'a>>,
) -> Result<(), store::error::Error> {
    let mut appender = cnx.appender_to_db(Tables::ConnectionDump.as_str(), "main")?;
    let mut traces_appender = cnx.appender_to_db(Tables::ConnectionDumpTraces.as_str(), "main")?;

    for entry in iter {
        match entry {
            ConnectionDumpEntry::Signal(Signal {
                cause,
                timestamp,
                tid,
                suppressed,
            }) => appender.append_row((
                timestamp,
                tid,
                None::<u64>,
                None::<u64>,
                None::<u64>,
                Some(cause.as_str()),
                Some(suppressed),
            ))?,
            ConnectionDumpEntry::Stats(Stats {
                tid,
                timestamp,
                used,
                free,
                total,
            }) => appender.append_row((
                timestamp,
                tid,
                Some(used),
                Some(free),
                Some(total),
                None::<&'static str>,
                None::<bool>,
            ))?,
            ConnectionDumpEntry::Trace(TraceDump {
                tid,
                timestamp,
                traces,
            }) => {
                appender.append_row((
                    timestamp,
                    tid,
                    None::<u64>,
                    None::<u64>,
                    None::<u64>,
                    None::<&'static str>,
                    None::<bool>,
                ))?;
                for trace in traces {
                    for (idx, frame) in (0..).zip(trace.stack_trace) {
                        traces_appender.append_row((
                            timestamp,
                            tid,
                            trace.duration,
                            trace.start_time,
                            idx,
                            frame,
                            trace.id,
                            &trace.invoked_by,
                            &trace.thread_name,
                        ))?;
                    }
                }
            }
        };
    }

    Ok(())
}

pub fn append_healthmeter<'a>(
    cnx: &Connection,
    iter: impl Iterator<Item = HCell<'a>>,
) -> Result<(), store::error::Error> {
    let mut healthmeter = cnx.appender_to_db(Tables::HealthMeter.as_str(), "main")?;
    healthmeter.append_rows(iter.map(|c| (c.key, c.val)))?;
    Ok(())
}

pub fn append_threaddump<'a>(
    cnx: &Connection,
    iter: impl Iterator<Item = ThreadDump<'a>>,
) -> Result<(), store::error::Error> {
    let mut threaddump = cnx.appender_to_db(Tables::Threaddump.as_str(), "main")?;
    let mut threads = cnx.appender_to_db(Tables::ThreaddumpThreads.as_str(), "main")?;
    let mut traces = cnx.appender_to_db(Tables::ThreaddumpTraces.as_str(), "main")?;

    for dump in iter {
        threaddump.append_row([dump.timestamp])?;
        for thread in dump.threads {
            let state = thread.state.as_str();
            let (object, lock, owner_id, owner_name) = match thread.state {
                crate::parser::threaddump::State::New
                | crate::parser::threaddump::State::Runnable
                | crate::parser::threaddump::State::Terminated => (None, None, None, None),
                crate::parser::threaddump::State::TimedWaiting { waiting_on } => {
                    (waiting_on.map(|s| s.0), None, None, None)
                }
                crate::parser::threaddump::State::Waiting {
                    waiting_on,
                    lock,
                    lock_owner_tid,
                    lock_owner_name,
                } => (
                    Some(waiting_on.0),
                    lock.map(|l| l.0),
                    lock_owner_tid,
                    lock_owner_name.map(|l| l.0),
                ),
                crate::parser::threaddump::State::Blocked {
                    blocked_on,
                    lock,
                    lock_owner_tid,
                    lock_owner_name,
                } => (
                    Some(blocked_on.0),
                    Some(lock.0),
                    Some(lock_owner_tid),
                    Some(lock_owner_name.0),
                ),
            };
            threads.append_row((
                dump.timestamp,
                thread.tid,
                thread.name,
                state,
                object,
                lock,
                owner_id,
                owner_name,
            ))?;

            if let Some(trace) = thread.trace {
                for (idx, frame) in (0..).zip(trace.0) {
                    let (method, source, object) = match frame {
                        Element::Lock { object } => (None, None, Some(object.0)),
                        Element::Frame { method, source } => (Some(method), Some(source), None),
                    };
                    traces.append_row((dump.timestamp, thread.tid, idx, method, source, object))?;
                }
            }
        }
    }

    Ok(())
}
