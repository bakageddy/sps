use duckdb::Appender;
use duckdb::Connection;
use duckdb::Result;

use std::path::Path;

use crate::parser::stacktrace::Element;
use crate::parser::stacktrace::Source;
use crate::parser::stacktrace::Trace;
use crate::parser::stuckquery_pgsql::StuckQueryTable;
use crate::parser::stuckthread::Begin;
use crate::parser::stuckthread::End;
use crate::parser::threaddump::ThreadDump;
use crate::parser::threaddump::ThreadState;

pub struct Store;
impl Store {
    pub fn open<P>(path: Option<P>) -> Result<Connection>
    where
        P: AsRef<Path>,
    {
        let cnx = if let Some(path) = path {
            Connection::open(path)?
        } else {
            Connection::open_in_memory()?
        };

        Ok(cnx)
    }

    pub fn schema(cnx: &Connection) -> Result<()> {
        let schema = include_str!("../../schema.duckdb.sql");
        cnx.execute_batch(schema)?;
        Ok(())
    }

    pub fn insert_stuckthread(
        stuckthread_appender: &mut Appender,
        stacktrace_appender: &mut Appender,
        begin: &Begin,
        trace: &Trace,
        end: Option<&End>,
    ) -> Result<()> {
        let mut active_monitor_end = None;
        let mut active_duration_ms = begin.active_duration_ms;
        if let Some(end) = end {
            active_monitor_end = Some(end.active_monitor_count);
            active_duration_ms = end.active_duration_ms;
        };
        let _ = stuckthread_appender.append_row((
            begin.tid,
            begin.start,
            active_duration_ms,
            begin.active_monitor_count,
            active_monitor_end,
            Some(begin.name),
            Some(begin.request),
        ))?;

        stacktrace_appender.append_rows((0..).zip(&trace.0).map(
            |(idx, element)| match element {
                Element::Lock(_) => unreachable!("Lock Information in stuckthread"),
                Element::Elem { method, source } => {
                    let (file, line) = match source {
                        Source::NativeMethod => (Some("NativeMethod"), None),
                        Source::UnknownSource => (Some("UnknownSource"), None),
                        Source::Filename { file, line } => (Some(*file), Some(*line)),
                        Source::Generated(inner) => (Some(*inner), None),
                    };
                    (
                        begin.tid,
                        begin.start,
                        line,
                        None::<u64>,
                        idx,
                        None::<&str>,
                        Some(*method),
                        file,
                    )
                }
            },
        ))?;

        Ok(())
    }

    pub fn insert_threaddump(
        thread_appender: &mut Appender,
        stacktrace_appender: &mut Appender,
        dump: &ThreadDump,
    ) -> Result<()> {
        let mut stacktraces = Vec::with_capacity(dump.threads.len());
        for thread in &dump.threads {
            let mut identity = None;
            let mut class = None;
            let mut owner_id = None;
            let mut owner = None;
            let state = match &thread.state {
                ThreadState::New => "NEW",
                ThreadState::Terminated => "TERMINATED",
                ThreadState::Runnable => "RUNNABLE",
                ThreadState::TimedWaiting => "TIMED_WAITING",
                ThreadState::BlockedToLock(lock) => {
                    let temp_lock = lock
                        .as_ref()
                        .expect("SAFETY: Blocked state will always have a lock");
                    class = Some(temp_lock.object.class);
                    identity = Some(temp_lock.object.identity);
                    owner_id = Some(temp_lock.owner_id);
                    owner = temp_lock.owner_name.and_then(|s| Some(s));
                    "BLOCKED"
                }
                ThreadState::TimedWaitingOn(object) => {
                    class = Some(object.class);
                    identity = Some(object.identity);
                    "TIMED_WAITING"
                }
                ThreadState::Waiting => "WAITING",
                ThreadState::WaitingOn(object) => {
                    class = Some(object.class);
                    identity = Some(object.identity);
                    "WAITING"
                }
                ThreadState::WaitingToLock(lock) => {
                    class = Some(lock.object.class);
                    identity = Some(lock.object.identity);
                    owner_id = Some(lock.owner_id);
                    owner = lock.owner_name.and_then(|s| Some(s));
                    "WAITING"
                }
            };
            let _ = thread_appender.append_row((
                thread.tid,
                dump.triggered,
                identity,
                owner_id,
                dump.snapshot,
                owner,
                class,
                thread.name,
                state,
            ));

            if let Some(trace) = &thread.stacktrace {
                stacktraces.push((thread.tid, trace));
            }
        }

        let _ = stacktrace_appender.append_rows(
            stacktraces
                .iter()
                .map(|(tid, trace)| {
                    (0..)
                        .zip(&trace.0)
                        .map(move |(idx, element)| match element {
                            Element::Lock(object) => (
                                tid,
                                dump.triggered,
                                None,
                                Some(object.identity),
                                idx,
                                Some(object.class),
                                None,
                                None,
                            ),
                            Element::Elem { method, source } => {
                                let (file, line) = match source {
                                    Source::NativeMethod => (Some("NativeMethod"), None),
                                    Source::UnknownSource => (Some("UnknownSource"), None),
                                    Source::Filename { file, line } => (Some(*file), Some(*line)),
                                    Source::Generated(inner) => (Some(*inner), None),
                                };
                                (
                                    tid,
                                    dump.triggered,
                                    line,
                                    None,
                                    idx,
                                    None,
                                    Some(*method),
                                    file,
                                )
                            }
                        })
                })
                .flatten(),
        )?;
        Ok(())
    }

    pub fn insert_stuckquery_pgsql_table(
        appender: &mut Appender,
        table: &StuckQueryTable,
    ) -> Result<()> {
        for query in &table.queries {
            let _ = appender.append_row((
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
            ))?;
        }
        Ok(())
    }
}
