use rusqlite::{MappedRows, OptionalExtension, Rows};
use std::collections::HashMap;
pub struct ThreadDump {
    pub threads: HashMap<i64, Thread>,
    pub triggered_unix_ms: i64,
    pub snapshot: u8,
}

pub struct Thread {
    pub trace: Option<Trace>,
    pub name: Option<String>,
    pub state: State,
    pub id: i64,
}

pub enum State {
    New,
    Terminated,
    Runnable,
    BlockedOn(Option<Lock>),
    Waiting,
    WaitingOn(Object),
    TimedWaitingOn(Object),
    WaitingToLock(Lock),
}

pub struct Trace(pub Vec<Frame>);

pub enum Frame {
    Frame { method: String, source: Source },
    Lock(Object),
}

pub enum Source {
    NativeMethod,
    UnknownSource,
    Generated(String),
    Filename { file: String, line: i64 },
}

pub struct Lock {
    pub name: Option<String>,
    pub object: Object,
    pub thread_id: i64,
}

pub struct Object {
    pub class: String,
    pub identity: i64,
}

pub struct StuckThread {
    pub trace: Trace,
    pub name: Option<String>,
    pub request: Option<String>,
    pub active_count_start: Option<i64>,
    pub active_count_end: Option<i64>,
    pub id: i64,
    pub active_duration_ms: i64,
    pub start: i64,
}

// NOTE: I am sure that i would change this.
impl<'a> IntoIterator for &'a Trace {
    type Item = &'a Frame;
    type IntoIter = std::slice::Iter<'a, Frame>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl StuckThread {
    pub fn row_mapper(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(
        i64,
        i64,
        Option<String>,
        Option<String>,
        i64,
        Option<i64>,
        Option<i64>,
        i64,
    )> {
        row.try_into()
    }

    pub fn get_by_range(
        cnx: &rusqlite::Connection,
        start: i64,
        end: i64,
    ) -> rusqlite::Result<Vec<StuckThread>> {
        let mut stmt = cnx.prepare_cached("SELECT * FROM stuckthread WHERE start BETWEEN ? AND ? UNION SELECT * FROM stuckthread WHERE start + active_duration_ms BETWEEN ? and ?;")?;
        let rows = stmt.query([start, end, start, end])?;
        StuckThread::extract_rows(cnx, rows)
    }

    pub fn get_by_id_start(
        cnx: &rusqlite::Connection,
        start: i64,
        id: i64,
    ) -> rusqlite::Result<Option<StuckThread>> {
        let mut stmt =
            cnx.prepare_cached("SELECT * FROM stuckthread WHERE start = ? AND id = ?;")?;
        match stmt
            .query_one([start, id], StuckThread::row_mapper)
            .optional()
        {
            Ok(Some((
                id,
                start,
                name,
                request,
                active_duration_ms,
                active_count_start,
                active_count_end,
                trace_id,
            ))) => {
                let trace = Trace::get_by_id(cnx, trace_id)?
                    .expect("Trace will always be valid from a stuckthread");
                Ok(Some(Self {
                    trace,
                    name,
                    request,
                    active_count_start,
                    active_count_end,
                    id,
                    active_duration_ms,
                    start,
                }))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn get_by_name(
        cnx: &rusqlite::Connection,
        name_pattern: String,
    ) -> rusqlite::Result<Vec<StuckThread>> {
        todo!()
    }

    pub fn get_by_request(
        cnx: &rusqlite::Connection,
        request_pattern: String,
    ) -> rusqlite::Result<Vec<StuckThread>> {
        todo!()
    }

    pub fn extract_rows(
        cnx: &rusqlite::Connection,
        rows: Rows<'_>,
    ) -> rusqlite::Result<Vec<StuckThread>> {
        let mut events = Vec::new();
        for row in rows.mapped(StuckThread::row_mapper) {
            let (
                id,
                start,
                name,
                request,
                active_duration_ms,
                active_count_start,
                active_count_end,
                stack_id,
            ) = row?;
            let trace = Trace::get_by_id(cnx, stack_id)?
                .expect("Trace from a stuckthread should always be valid");
            events.push(Self {
                trace,
                name,
                request,
                active_duration_ms,
                id,
                active_count_start,
                active_count_end,
                start,
            });
        }
        Ok(events)
    }
}

// Convenience functions all the way down.
impl Trace {
    pub fn row_mapper(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(
        i64,
        i64,
        Option<String>,
        Option<String>,
        Option<i64>,
        Option<i64>,
    )> {
        row.try_into()
    }

    pub fn exists(cnx: &rusqlite::Connection, id: i64) -> rusqlite::Result<bool> {
        let mut stmt = cnx.prepare_cached("SELECT 1 FROM stacktrace WHERE id=?")?;
        stmt.exists((id,))
    }

    pub fn get_by_id(cnx: &rusqlite::Connection, id: i64) -> rusqlite::Result<Option<Trace>> {
        let mut stmt = cnx.prepare_cached("SELECT stacktrace_elements.* FROM stacktrace INNER JOIN stacktrace_elements ON stacktrace.id = stacktrace_elements.id WHERE stacktrace.id = ? ORDER BY stacktrace_elements.frame_idx")?;
        let rows = stmt.query([id])?;
        let mut frames = Vec::with_capacity(20);
        for row in rows.mapped(Trace::row_mapper) {
            let (_, _, method, source, line, object_id) = row?;
            let frame = if let Some(object_id) = object_id {
                let object = Object::get_by_id(cnx, object_id)?
                    .expect("Database provided object_id does not exists");
                Frame::Lock(object)
            } else {
                let method = method.expect("Database provided method is null");
                let source = Source::from(source, line)
                    .expect("Database provided source and line_number combo is not valid");
                Frame::Frame { method, source }
            };
            frames.push(frame);
        }
        Ok(Some(Trace(frames)))
    }
}

impl Source {
    pub fn from(source: Option<String>, line: Option<i64>) -> Option<Source> {
        match (source.as_deref(), line) {
            (Some("NativeMethod"), _) => Some(Source::NativeMethod),
            (Some("UnknownSource"), _) => Some(Source::UnknownSource),
            (Some(_), None) => Some(Source::Generated(source.expect("SAFETY: Checked in match"))),
            (Some(_), Some(_)) => Some(Source::Filename {
                file: source.expect("SAFETY: Checked in match"),
                line: line.expect("SAFETY: Checked in match"),
            }),
            _ => None,
        }
    }
}

impl Object {
    pub fn row_mapper(val: &rusqlite::Row) -> rusqlite::Result<(i64, String, i64)> {
        val.try_into()
    }
    pub fn get_by_id(cnx: &rusqlite::Connection, id: i64) -> rusqlite::Result<Option<Object>> {
        let mut stmt = cnx.prepare_cached("SELECT id, class, identity FROM object WHERE id = ?")?;
        match stmt.query_one([id], Object::row_mapper).optional() {
            Ok(Some((_, class, identity))) => Ok(Some(Object { class, identity })),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
