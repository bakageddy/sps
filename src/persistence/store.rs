use duckdb::Appender;
use duckdb::Connection;
use duckdb::Result;

use std::path::Path;

use crate::parser::stacktrace::Element;
use crate::parser::stacktrace::Source;
use crate::parser::stacktrace::Trace;
use crate::parser::stuckthread::Begin;
use crate::parser::stuckthread::End;

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
            str::from_utf8(begin.name).ok(),
            str::from_utf8(begin.request).ok(),
        ))?;

        stacktrace_appender.append_rows((0..).zip(&trace.0).map(|(idx, element)| {
            match element {
                Element::Lock(_) => panic!("Unreachable: Lock Information in stuckthread"),
                Element::Elem { method, source } => {
                    let (file, line) = match source {
                        Source::NativeMethod => (Some("NativeMethod"), None),
                        Source::UnknownSource => (Some("UnknownSource"), None),
                        Source::Filename { file, line } => (str::from_utf8(file).ok(), Some(line)),
                        Source::Generated(inner) => (str::from_utf8(inner).ok(), None),
                    };
                    (
                        begin.tid,
                        begin.start,
                        line,
                        None::<u64>,
                        idx,
                        None::<&str>,
                        str::from_utf8(*method).ok(),
                        file,
                    )
                }
            }
        }))?;

        Ok(())
    }
}
