use crate::handlers::types::{ThreadDumpSummary, ThreadDumpThread};
use crate::parser::threaddump::{Element, Object};
use crate::store::{error::Error, tables::Tables};
use duckdb::Connection;

pub fn get_threaddump_summary(cnx: &Connection) -> Result<Vec<ThreadDumpSummary>, Error> {
    let query = format!(
        "SELECT timestamp, COUNT(tid), COUNT(tid) FILTER (WHERE state = 'RUNNABLE'), COUNT(tid) FILTER (WHERE state = 'BLOCKED'), COUNT(tid) FILTER (WHERE state = 'WAITING'), COUNT(tid) FILTER (WHERE state = 'TIMED_WAITING') FROM {0} GROUP BY timestamp ORDER BY timestamp",
        Tables::ThreaddumpThreads
    );
    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([])?;
    let mut summaries = Vec::new();
    while let Some(row) = rows.next()? {
        summaries.push(ThreadDumpSummary {
            timestamp: row.get(0)?,
            threads: row.get(1)?,
            runnable: row.get(2)?,
            blocked: row.get(3)?,
            waiting: row.get(4)?,
            timed_waiting: row.get(5)?,
        });
    }
    Ok(summaries)
}

pub fn get_threaddump(cnx: &Connection, timestamp: u64) -> Result<Vec<ThreadDumpThread>, Error> {
    let query = format!(
        r"SELECT 
            tid,
            name,
            state::VARCHAR,
            object,
            lock, 
            owner_tid, 
            owner_name,
            EXISTS(
                SELECT 1
                FROM {1}
                WHERE {0}.timestamp = {1}.timestamp
                AND {0}.tid = {1}.tid
            )
        FROM {0} 
        WHERE timestamp = $1",
        Tables::ThreaddumpThreads,
        Tables::ThreaddumpTraces,
    );
    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([timestamp])?;
    let mut threads = Vec::new();
    while let Some(row) = rows.next()? {
        threads.push(ThreadDumpThread {
            tid: row.get(0)?,
            name: row.get(1)?,
            state: row.get(2)?,
            waiting_on: row.get(3)?,
            lock: row.get(4)?,
            lock_owner_tid: row.get(5)?,
            lock_owner_name: row.get(6)?,
            has_trace: row.get(7)?,
        });
    }
    Ok(threads)
}

pub fn get_thread_trace(
    cnx: &Connection,
    timestamp: u64,
    tid: u64,
) -> Result<Vec<Element<'static>>, Error> {
    let query = format!(
        "SELECT method, source, object FROM {0} WHERE timestamp = $1 AND tid = $2 ORDER BY idx",
        Tables::ThreaddumpTraces
    );
    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([timestamp, tid])?;
    let mut frames = Vec::new();
    while let Some(row) = rows.next()? {
        let object: Option<String> = row.get(2)?;
        if let Some(object) = object {
            frames.push(Element::Lock {
                object: Object(object.into()),
            });
        } else {
            let method: String = row.get(0)?;
            let source: String = row.get(1)?;
            frames.push(Element::Frame {
                method: method.into(),
                source: source.into(),
            });
        }
    }
    Ok(frames)
}
