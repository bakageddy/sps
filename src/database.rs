use std::path::PathBuf;

use tracing::warn;

use crate::{
    stacktrace::{self},
    stuckthread::{StuckThread, StuckThreadMeta},
    threaddump::{self, Object, Thread, ThreadDump, ThreadState},
    util::{self, ToUnixMillis},
};

pub struct Persistence;

impl Persistence {
    // TODO: bake in schema into the executable
    pub fn init_db(path: Option<PathBuf>) -> util::Result<rusqlite::Connection> {
        let schema = include_str!("../schema.sql");

        let cnx = if let Some(path) = path {
            rusqlite::Connection::open(path)?
        } else {
            rusqlite::Connection::open_in_memory()?
        };

        cnx.execute_batch(schema)?;
        cnx.execute_batch(
            r#"PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL; PRAGMA cache_size = -64000; PRAGMA temp_store = MEMORY;"#,
        )?;
        Ok(cnx)
    }

    pub fn insert_stacktrace(tx: &rusqlite::Transaction) -> rusqlite::Result<i64> {
        let mut stmt = tx.prepare_cached("INSERT INTO stacktrace DEFAULT VALUES RETURNING id")?;
        let id: i64 = stmt.query_one([], |r| r.get(0))?;
        Ok(id)
    }
 
    pub fn insert_stuckthread_stacktrace(
        tx: &rusqlite::Transaction,
        stack: &crate::stacktrace::StackTrace,
    ) -> rusqlite::Result<i64> {
        let id = Persistence::insert_stacktrace(tx)?;
        let mut elems = tx.prepare_cached(
            "INSERT INTO stacktrace_elements(id, frame_idx, method, frame_source, line_number, object_id) VALUES(?, ?, ?, ?, ?, NULL);"
        )?;

        for (i, elem) in (0..).zip(stack.traces.iter()) {
            let (frame_source, line_number) = match elem.stacktrace_source {
                stacktrace::StackTraceSource::NativeMethod => ("NativeMethod", None),
                stacktrace::StackTraceSource::UnknownSource => ("UnknownSource", None),
                stacktrace::StackTraceSource::FileName { file, line } => (file, Some(line as i64)),
                stacktrace::StackTraceSource::Generated { inner } => (inner, None),
            };

            elems.insert((id, i, elem.function_name, frame_source, line_number))?;
        }

        Ok(id)
    }

    pub fn insert_stuckthread(
        tx: &rusqlite::Transaction,
        begin: &StuckThread,
        end: Option<&StuckThread>,
    ) -> rusqlite::Result<()> {
        let stack_id = Persistence::insert_stuckthread_stacktrace(
            tx,
            begin.st.as_ref().expect("REQUIRE: metabegin in begin"),
        )?;
        let mut stmt = tx.prepare_cached(
            "INSERT INTO stuckthread(thread_id, start, name, request, active_duration_ms, active_monitor_start, active_monitor_end, stack_id) VALUES(?, ?, ?, ?, ?, ?, ?, ?);"
        )?;

        let begin = match &begin.meta {
            StuckThreadMeta::Begin(x) => x,
            _ => panic!("UNREACHABLE: Expect StuckThreadMetaBegin not StuckThreadEnd"),
        };

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
        let start = begin.start.to_unix_millis();
        let active_monitor_count_begin = begin.active_monitor_count;
        let mut active_monitor_count_end = 0;
        let mut active_duration_count = begin.active_duration_ms;

        if let Some(end) = end && let StuckThreadMeta::End(end) = &end.meta {
            active_monitor_count_end = end.active_monitor_count;
            active_duration_count = end.active_duration_ms;
        }

        let _ = stmt.insert((
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

    pub fn insert_threaddump(
        tx: &rusqlite::Transaction,
        item: &ThreadDump,
    ) -> rusqlite::Result<i64> {
        let mut stmt = tx.prepare_cached(
            "INSERT INTO threaddump(snapshot, triggered_unix_ms) VALUES(?, ?) RETURNING id;"
        )?;
        let threaddump_id: i64 = stmt.query_one((item.snapshot, item.triggered_unix_ms), |r| r.get(0))?;
        for thread in item.threads.values() {
            if let Err(e) = Persistence::insert_thread(tx, thread, threaddump_id) {
                warn!(
                    "Error persisting thread {thread:?} for snapshot {}: {e:?}",
                    item.snapshot
                );
            }
        }
        Ok(threaddump_id)
    }

    pub fn insert_thread(
        tx: &rusqlite::Transaction,
        item: &Thread,
        threaddump_id: i64
    ) -> rusqlite::Result<()> {
        let mut stmt = tx.prepare_cached(
            "INSERT INTO thread(id, name, state, wait_object_id, owner_id, owner_name, lock_object_id, stack_id, dump_id) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?);"
        )?;
        let mut lock_info = None;
        let mut owner_id = None;
        let mut owner_name = None;
        let mut wait_object_id = None;
        let mut stack_id = None;

        if let Some(stack) = item.stacktrace.as_ref() {
            let result = Persistence::insert_thread_stacktrace(tx, stack)?;
            stack_id = Some(result);
        };

        let state = match &item.state {
            ThreadState::New => "NEW",
            ThreadState::Terminated => "TERMINATED",
            ThreadState::Runnable => "RUNNABLE",
            ThreadState::TimedWaiting => "TIMED_WAITING",
            ThreadState::BlockedToLock(lock) => {
                let temp_lock = lock
                    .as_ref()
                    .expect("SAFETY: Blocked state will always have a lock");
                let object_id = Persistence::insert_thread_object(tx, &temp_lock.object)?;
                lock_info = Some(object_id);
                owner_id = Some(temp_lock.owner_id);
                owner_name = temp_lock.owner_name;
                "BLOCKED"
            }
            ThreadState::TimedWaitingOn(object) => {
                let object_id = Persistence::insert_thread_object(tx, object)?;
                wait_object_id = Some(object_id);
                "TIMED_WAITING"
            }
            ThreadState::Waiting => "WAITING",
            ThreadState::WaitingOn(object) => {
                let object_id = Persistence::insert_thread_object(tx, object)?;
                wait_object_id = Some(object_id);
                "WAITING"
            }
            ThreadState::WaitingToLock(lock) => {
                let object_id = Persistence::insert_thread_object(tx, &lock.object)?;
                owner_id = Some(lock.owner_id);
                owner_name = lock.owner_name;
                lock_info = Some(object_id);
                "WAITING"
            }
        };

        let _ = stmt.insert((
            item.thread_id,
            &item.thread_name,
            state,
            wait_object_id,
            owner_id,
            owner_name,
            lock_info,
            stack_id,
            threaddump_id
        ))?;

        Ok(())
    }

    pub fn insert_thread_object(
        tx: &rusqlite::Transaction,
        item: &Object,
    ) -> rusqlite::Result<i64> {
        let mut stmt =
            tx.prepare_cached("INSERT INTO object(class, identity) VALUES(?, ?) RETURNING id;")?;
        let id: i64 = 0;
        let (id, overflow) = id.overflowing_add_unsigned(item.identity);
        if overflow {
            warn!("Overflow while converting object id: {id}");
        }
        let obj_id: i64 = stmt.query_row((item.class, id), |s| s.get(0))?;
        Ok(obj_id)
    }

    pub fn insert_thread_stacktrace(
        tx: &rusqlite::Transaction,
        item: &threaddump::StackTrace,
    ) -> rusqlite::Result<i64> {
        let stackid = Persistence::insert_stacktrace(tx)?;

        let mut stmt = tx.prepare_cached("INSERT INTO stacktrace_elements(id, frame_idx, method, frame_source, line_number, object_id) VALUES(?, ?, ?, ?, ?, ?);")?;
        for (idx, elem) in std::iter::zip(0.., item.elems.iter()) {
            match elem {
                threaddump::Element::Lock(object) => {
                    let object_id = Persistence::insert_thread_object(tx, object)?;
                    let _ = stmt.insert((
                        stackid,
                        idx,
                        None::<&str>,
                        None::<&str>,
                        None::<i64>,
                        Some(object_id),
                    ))?;
                }
                threaddump::Element::Elem { frame, source } => {
                    let (frame_source, line_number) = match source {
                        crate::threaddump::Source::NativeMethod => ("NativeMethod", None),
                        crate::threaddump::Source::UnknownSource => ("UnknownSource", None),
                        crate::threaddump::Source::Generated(inner) => (*inner, None),
                        crate::threaddump::Source::Filename { file, line_number } => {
                            (*file, Some(line_number))
                        }
                    };
                    let _ = stmt.insert((
                        stackid,
                        idx,
                        Some(*frame),
                        Some(frame_source),
                        line_number,
                        None::<i64>,
                    ));
                }
            };
        }
        Ok(stackid)
    }
}
