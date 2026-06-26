use duckdb::{Appender, DuckdbConnectionManager, Result, params};
use r2d2::{Pool, PooledConnection};

use std::path::Path;

use crate::parser::{
    cpumemstats_windows::CPUMemoryStats,
    cpumonitoring::CPUThread,
    stacktrace::{Element, Trace},
    stuckquery_mssql::{self, Status},
    stuckquery_pgsql,
    stuckthread::{Event, StuckThread},
    threaddump::ThreadDump,
};
use crate::util::{self, SchemaMapper};

pub struct Store;
impl Store {
    pub fn open<P>(path: Option<P>) -> util::Result<Pool<DuckdbConnectionManager>>
    where
        P: AsRef<Path>,
    {
        let mgr = if let Some(path) = path {
            DuckdbConnectionManager::file(path)?
        } else {
            DuckdbConnectionManager::memory()?
        };

        let pool = r2d2::Pool::new(mgr)?;

        Ok(pool)
    }

    pub fn schema(cnx: &PooledConnection<DuckdbConnectionManager>) -> Result<()> {
        let schema = include_str!("../../schema.duckdb.sql");
        cnx.execute_batch(schema)?;
        Ok(())
    }

    pub fn insert_stuckthread<'a>(
        stuckthread_appender: &mut Appender,
        stacktrace_appender: &mut Appender,
        iter: Vec<StuckThread<'a>>,
    ) -> Result<()> {
        for event in iter {
            match &event.0 {
                Event::Begin(begin, trace) => Self::insert_stuckthread_stacktrace(
                    stacktrace_appender,
                    begin.tid,
                    begin.start,
                    trace,
                )?,
                _ => {}
            };
            stuckthread_appender.append_row(event.map_to_row())?;
        }
        Ok(())
    }

    pub fn insert_stuckthread_stacktrace(
        appender: &mut Appender,
        tid: u64,
        stamp: u64,
        trace: &Trace,
    ) -> Result<()> {
        for (idx, item) in (0u32..).zip(&trace.0) {
            match item {
                Element::Lock(_) => {
                    unreachable!("Stuckthreads' stacktraces do not have locked objects");
                }
                Element::Elem { method, source } => {
                    let mapped = source.map_to_row();
                    appender.append_row((tid, stamp, mapped.1, idx, *method, mapped.0))?;
                }
            }
        }
        Ok(())
    }


    pub fn insert_stuckquery_pgsql_table(
        appender: &mut Appender,
        table: &stuckquery_pgsql::StuckQueryTable,
    ) -> Result<()> {
        appender.append_rows(table.queries.iter().map(|query| {
            (
                table.timestamp,
                query.pid,
                query.query_time_ms,
                query.txn_time_ms,
                query.last_state_change,
                query.client_port,
                query.state.is_active(),
                query.waiting,
                query.client_address.map(|x| x.to_string()),
                Some(query.db_name),
                Some(query.query),
                Some(query.application_name),
                query.client_hostname,
            )
        }))
    }

    pub fn insert_stuckquery_mssql_table(
        appender: &mut Appender,
        table: &stuckquery_mssql::StuckQueryTable,
    ) -> Result<()> {
        for query in &table.queries {
            let status = match query.status {
                Status::Running => "RUNNING",
                Status::Runnable => "RUNNABLE",
                Status::Suspended => "SUSPENDED",
            };
            let _ = appender.append_row(params![
                table.timestamp,
                query.session_id,
                status,
                query.txn_id,
                query.blocked_by,
                query.wait_type,
                query.wait_resource,
                query.wait_time_ms,
                query.cpu_time_ms,
                query.logical_reads,
                query.physical_reads,
                query.physical_writes,
                query.elapsed_time_ms,
                query.statement,
                query.command_text,
                query.command,
                query.login_name,
                query.host_name,
                query.db_name,
                query.program_name,
                query.host_process_id,
                query.last_request_end_ms,
                query.login_time_ms,
                query.open_transaction_count,
            ])?;
        }
        Ok(())
    }

    pub fn insert_cpumonitoring_threads(
        cpu_appender: &mut Appender,
        stacktrace_appender: &mut Appender,
        threads: Vec<CPUThread>,
    ) -> Result<()> {
        for thread in threads {
            if let Some(trace) = thread.trace {
                Store::insert_cpumonitoring_stacktrace(
                    stacktrace_appender,
                    thread.tid,
                    thread.timestamp,
                    trace,
                )?;
            }
            cpu_appender.append_row((
                thread.tid,
                thread.timestamp,
                Some(thread.name),
                thread.state.to_str(),
                thread.cpu,
            ))?;
        }
        Ok(())
    }

    pub fn insert_cpumonitoring_stacktrace(
        appender: &mut Appender,
        tid: u64,
        stamp: u64,
        trace: Trace,
    ) -> Result<()> {
        for (idx, item) in (0u32..).zip(&trace.0) {
            match item {
                Element::Lock(_) => {
                    unreachable!("CPUMonitoring stacktraces do not have locked objects");
                }
                Element::Elem { method, source } => {
                    let mapped = source.map_to_row();
                    appender.append_row((tid, stamp, mapped.1, idx, *method, mapped.0))?;
                }
            }
        }
        Ok(())
    }

    pub fn insert_threaddump(
        thread_appender: &mut Appender,
        stacktrace_appender: &mut Appender,
        dump: ThreadDump,
    ) -> Result<()> {
        for thread in &dump.threads {
            thread_appender.append_row(thread.map_to_row())?;
            if let Some(ref trace) = thread.stacktrace {
                Store::insert_threaddump_stacktrace(
                    stacktrace_appender,
                    trace,
                    thread.tid,
                    dump.triggered,
                )?;
            }
        }
        Ok(())
    }

    pub fn insert_threaddump_stacktrace(
        appender: &mut Appender,
        trace: &Trace,
        tid: u64,
        timestamp: u64,
    ) -> Result<()> {
        appender.append_rows(trace.0.iter().zip(0..).map(|(item, idx)| {
            let mapped = item.map_to_row();
            (
                tid, timestamp, mapped.0, mapped.1, idx, mapped.2, mapped.3, mapped.4,
            )
        }))
    }

    pub fn insert_cpumemstats(appender: &mut Appender, stats: Vec<CPUMemoryStats>) -> Result<()> {
        for table in stats {
            match table {
                CPUMemoryStats::CPU(table) => {
                    appender.append_rows(table.stats.iter().map(|stat| stat.map_to_row()).map(
                        |(pid, usage, path, name, is_cpu)| {
                            (
                                table.timestamp_ms,
                                pid,
                                table.total_cpu,
                                usage,
                                path,
                                name,
                                is_cpu,
                            )
                        },
                    ))?;
                }
                CPUMemoryStats::Memory(table) => {
                    appender.append_rows(table.stats.iter().map(|stat| stat.map_to_row()).map(
                        |(pid, usage, path, name, is_cpu)| {
                            (
                                table.timestamp_ms,
                                pid,
                                table.total_memory,
                                usage,
                                path,
                                name,
                                is_cpu,
                            )
                        },
                    ))?;
                }
            }
        }
        Ok(())
    }
}
