use duckdb::{Connection, types::Value};

use crate::{
    handlers::types::{ConnectionDumpHolder, ConnectionDumpSnapshot},
    parser::connectiondump::{Signal, Stats, Trace},
    store::{error::Error, tables::Tables},
};

pub fn get_connectiondump_signals(
    cnx: &Connection,
    from: Option<u64>,
    to: Option<u64>,
) -> Result<Vec<Signal>, Error> {
    let query = format!(
        "SELECT timestamp, tid, cause::VARCHAR, suppressed FROM {0} WHERE timestamp BETWEEN $1 AND $2 AND cause IS NOT NULL ORDER BY timestamp",
        Tables::ConnectionDump
    );

    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([from.unwrap_or(u64::MIN), to.unwrap_or(u64::MAX)])?;
    let mut result = Vec::new();
    while let Some(row) = rows.next()? {
        result.push(Signal {
            timestamp: row.get(0)?,
            tid: row.get(1)?,
            cause: row.get(2)?,
            suppressed: row.get(3)?,
        });
    }
    Ok(result)
}

pub fn get_connectiondump_stats(
    cnx: &Connection,
    from: Option<u64>,
    to: Option<u64>,
) -> Result<Vec<Stats>, Error> {
    let query = format!(
        "SELECT timestamp, tid, used, free, total FROM {0} WHERE timestamp BETWEEN $1 AND $2 AND used IS NOT NULL AND free IS NOT NULL ORDER BY timestamp",
        Tables::ConnectionDump
    );
    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([from.unwrap_or(u64::MIN), to.unwrap_or(u64::MAX)])?;
    let mut result = Vec::new();
    while let Some(row) = rows.next()? {
        result.push(Stats {
            timestamp: row.get(0)?,
            tid: row.get(1)?,
            used: row.get(2)?,
            free: row.get(3)?,
            total: row.get(4)?,
        });
    }
    Ok(result)
}

pub fn get_connectiondump_snapshots(
    cnx: &Connection,
) -> Result<Vec<ConnectionDumpSnapshot>, Error> {
    let query = format!(
        r"
SELECT
  stats.timestamp,
  stats.tid,
  stats.used,
  stats.free,
  stats.total,
  cnx_traces.trace_count,
  cnx_traces.max_duration
FROM
  (SELECT timestamp, tid, COUNT(DISTINCT id) as trace_count, MAX(duration) as max_duration FROM {0} GROUP BY timestamp, tid ORDER BY timestamp) cnx_traces 
  ASOF INNER JOIN (SELECT timestamp, tid, used, free, total FROM {1} WHERE used IS NOT NULL AND cause IS NULL ORDER BY timestamp) stats ON stats.tid = cnx_traces.tid
  AND stats.timestamp <= cnx_traces.timestamp
ORDER BY
  stats.timestamp
",
        Tables::ConnectionDumpTraces,
        Tables::ConnectionDump
    );
    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([])?;
    let mut results = Vec::new();
    while let Some(row) = rows.next()? {
        results.push(ConnectionDumpSnapshot {
            timestamp: row.get(0)?,
            used: row.get(2)?,
            total: row.get(4)?,
            trace_count: row.get(5)?,
            max_duration: row.get(6)?,
        });
    }
    Ok(results)
}

pub fn get_connectiondump_traces(
    cnx: &Connection,
    timestamp: u64,
) -> Result<Vec<Trace<'static>>, Error> {
    let query = format!(
        r"
        SELECT
            duration,
            start_time,
            id,
            LIST(frame ORDER BY idx),
            invoked_by,
            thread_name
        FROM {0}
        WHERE {0}.timestamp = $1
        GROUP BY timestamp, id, duration, start_time, invoked_by, thread_name",
        Tables::ConnectionDumpTraces
    );
    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([timestamp])?;
    let mut traces = Vec::new();
    while let Some(row) = rows.next()? {
        let value: Value = row.get(2)?;
        let stack_trace = match value {
            Value::List(items) => items
                .into_iter()
                .flat_map(|v| match v {
                    Value::Text(s) => Ok(s.into()),
                    _ => Err(()),
                })
                .collect(),
            _ => {
                return Err(Error::DuckDB(duckdb::Error::InvalidColumnType(
                    2,
                    "stacktraces.frames".to_owned(),
                    duckdb::types::Type::List(Box::new(duckdb::types::Type::Text)),
                )));
            }
        };
        traces.push(Trace {
            duration: row.get(0)?,
            start_time: row.get(1)?,
            stack_trace,
            id: row.get(3)?,
            invoked_by: row.get::<usize, Option<String>>(4)?.map(|s| s.into()),
            thread_name: row.get::<usize, String>(5)?.into(),
        });
    }
    Ok(traces)
}

pub fn get_connectiondump_holders(
    cnx: &Connection,
    from: Option<u64>,
    to: Option<u64>,
) -> Result<Vec<ConnectionDumpHolder>, Error> {
    let query = format!(
        r"
            WITH per_dump AS (
                SELECT
                    timestamp,
                    id,
                    start_time,
                    MAX(duration) AS duration,
                    LIST (frame ORDER BY idx) AS frames
                 FROM {0}
                 WHERE timestamp >= $1 AND timestamp <= $2
                 GROUP BY timestamp, id, start_time
            )
            SELECT
                id,
                start_time,
                MAX(duration) AS duration,
                COUNT(*) AS dump_count,
                ANY_VALUE (frames) AS stack_trace
            FROM per_dump
            GROUP BY id, start_time
            HAVING COUNT(*) >= 2",
        Tables::ConnectionDumpTraces
    );

    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([from.unwrap_or(u64::MIN), to.unwrap_or(u64::MAX)])?;
    let mut holders = Vec::new();
    while let Some(row) = rows.next()? {
        let value: Value = row.get(4)?;
        let stack_trace = match value {
            Value::List(items) => items
                .into_iter()
                .flat_map(|v| match v {
                    Value::Text(s) => Ok(s),
                    _ => Err(()),
                })
                .collect(),
            _ => {
                return Err(Error::DuckDB(duckdb::Error::InvalidColumnType(
                    2,
                    "stacktraces.frames".to_owned(),
                    duckdb::types::Type::List(Box::new(duckdb::types::Type::Text)),
                )));
            }
        };

        holders.push(ConnectionDumpHolder {
            id: row.get(0)?,
            start_time: row.get(1)?,
            duration: row.get(2)?,
            dump_count: row.get(3)?,
            stack_trace,
        });
    }
    Ok(holders)
}
