use std::{fs, path::PathBuf};

use crate::{
    stacktrace::{self, StackTrace}, stuckthread::{MetaBegin, MetaEnd}, util::{self, ToUnixMillis}
};

pub struct Persistence;

impl Persistence {
    // TODO: bake in schema into the executable
    pub fn init_db<P>(path: Option<PathBuf>, schema: P) -> util::Result<rusqlite::Connection> where P: AsRef<std::path::Path> {
        let schema = schema.as_ref();
        let schema = fs::read_to_string(schema)?;

        let cnx;
        if path.is_some() {
            cnx = rusqlite::Connection::open(path.expect("SAFETY: checked"))?;
        } else {
            cnx = rusqlite::Connection::open_in_memory()?;
        }

        cnx.execute_batch(&schema)?;
        Ok(cnx)
    }

    pub fn insert_stuckthread(
        tx: &rusqlite::Transaction,
        begin: &MetaBegin,
        stacktrace: &StackTrace,
        end: Option<&MetaEnd>,
    ) -> util::Result<()> {
        let mut pstmt = tx.prepare(
            "INSERT INTO stuckthread_meta(thread_id, start, thread_name, api_request, active_duration_ms, active_monitor_count_start, active_monitor_count_end) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING stack_id"
        )?;

        let thread_id = begin.thread_id;
        let thread_name = if begin.thread_name.is_empty() { None } else { Some(begin.thread_name) };
        let api_request = if begin.request.is_empty() { None } else { Some(begin.request) };
        let start = begin.start.to_unix_millis();
        let active_monitor_count_begin = begin.active_monitor_count;
        let mut active_monitor_count_end = 0;
        let mut active_duration_count = begin.active_duration_ms;

        if end.is_some() {
            active_monitor_count_end = end.expect("SAFETY: checked").active_monitor_count;
            active_duration_count = end.expect("SAFETY: checked").active_duration_ms;
        }

        let stack_id = pstmt.query_row(
            (thread_id, start, thread_name, api_request, active_duration_count, active_monitor_count_begin, active_monitor_count_end),
            |row| row.get::<_, i64>(0)
        )?;

        Persistence::insert_stacktrace(tx, stacktrace, stack_id)?;
        Ok(())
    }

    pub fn insert_stacktrace(tx: &rusqlite::Transaction, item: &StackTrace, stack_id: i64) -> rusqlite::Result<()> {
        let mut pstmt = 
            tx.prepare("INSERT INTO stuckthread_stack(stack_id, frame_idx, method, frame_source, line_number) VALUES(?, ?, ?, ?, ?);")?;
        let mut i = 0;
        for frame in &item.traces {
            let method = frame.function_name;
            let (frame_source, line_number) = match frame.stacktrace_source {
                stacktrace::StackTraceSource::NativeMethod => ("NativeMethod", None),
                stacktrace::StackTraceSource::UnknownSource => ("UnknownSource", None),
                stacktrace::StackTraceSource::FileName { file, line } => (file, Some(line as i64)),
                stacktrace::StackTraceSource::Generated { inner } => (inner, None),
            };

            pstmt.execute((stack_id, i, method, frame_source, line_number))?;
            i += 1;
        }
        Ok(())
    }
}
