use duckdb::Appender;
use duckdb::Connection;
use duckdb::Result;
use duckdb::Transaction;

use std::path::Path;

use crate::stacktrace::{self, StackTrace};
use crate::stuckthread::Begin;
use crate::stuckthread::End;
use crate::util::ToUnixMillis;

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

    pub fn insert_stacktrace(tx: &Transaction) -> Result<u64> {
        let query = "INSERT INTO stacktrace DEFAULT VALUES RETURNING id";
        let mut stmt = tx.prepare_cached(query)?;
        stmt.query_one([], |row| row.get(0))
    }

    pub fn insert_stacktrace_stuckthread(
        tx: &Transaction,
        appender: &mut Appender,
        stacktrace: &StackTrace,
    ) -> Result<u64> {
        let id = Store::insert_stacktrace(tx)?;
        // let mut appender = tx.appender("stacktrace_elements")?;
        (1..)
            .zip(stacktrace.traces.iter())
            .map(|(idx, element)| {
                let (frame_source, line_number) = match element.source {
                    stacktrace::StackTraceSource::NativeMethod => ("NativeMethod", None),
                    stacktrace::StackTraceSource::UnknownSource => ("UnknownSource", None),
                    stacktrace::StackTraceSource::FileName { file, line } => {
                        (file, Some(line as i64))
                    }
                    stacktrace::StackTraceSource::Generated { inner } => (inner, None),
                };
                (
                    id,
                    idx,
                    element.method,
                    frame_source,
                    line_number,
                    None::<u64>,
                )
            })
            .try_for_each(|extracted| appender.append_row(extracted))?;
        Ok(id)
    }

    pub fn insert_stuckthread(
        stuckthread_appender: &mut Appender,
        tx: &Transaction,
        stacktrace_elements_appender: &mut Appender,
        begin: &Begin,
        st: &StackTrace,
        end: Option<&End>,
    ) -> Result<()> {
        let stack_id = Store::insert_stacktrace_stuckthread(
            tx,
            stacktrace_elements_appender,
            st,
        )?;
        let thread_id = begin.thread_id;
        let thread_name = if begin.thread_name.is_empty() {
            None
        } else {
            Some(begin.thread_name)
        };
        let api_request = if begin.request.is_empty() {
            None
        } else {
            Some(begin.request)
        };
        let start = begin.start.to_unix_millis().map(|s| s as u64);
        let active_monitor_count_begin = begin.active_monitor_count;
        let mut active_monitor_count_end = 0;
        let mut active_duration_count = begin.active_duration_ms;

        if let Some(end) = end {
            active_monitor_count_end = end.active_monitor_count;
            active_duration_count = end.active_duration_ms;
        }

        stuckthread_appender.append_row((
            thread_id,
            start,
            thread_name,
            api_request,
            active_duration_count,
            active_monitor_count_begin,
            active_monitor_count_end,
            stack_id,
        ))?;

        Ok(())
    }
}
