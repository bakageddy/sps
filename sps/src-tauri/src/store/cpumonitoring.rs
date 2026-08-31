use crate::handlers::types::{CPUPoint, DumpSummary};
use crate::store::error::Error;
use duckdb::Connection;

use crate::handlers::types::CPUThread;
use crate::store::tables::Tables;
use crate::store::types::Frame;

pub fn get_stackframes(
    cnx: &Connection,
    tid: u64,
    timestamp: u64,
) -> Result<Option<Vec<Frame>>, Error> {
    let query = format!(
        "SELECT method, source FROM {} WHERE tid=$1 AND timestamp=$2",
        Tables::CPUMonitoringStackTraces.into_str()
    );
    let mut stmt = cnx.prepare_cached(&query)?;
    let mut result = stmt.query([tid, timestamp])?;
    let mut frames = Vec::new();
    while let Some(row) = result.next()? {
        let frame = Frame {
            method: row.get(0)?,
            source: row.get(1)?,
        };
        frames.push(frame);
    }

    if frames.is_empty() {
        Ok(None)
    } else {
        Ok(Some(frames))
    }
}

pub fn get_cpu_dumps(cnx: &Connection) -> Result<Vec<DumpSummary>, Error> {
    let query = format!(
        "SELECT timestamp, COUNT(DISTINCT tid), MAX(cpu), SUM(cpu) FROM {} GROUP BY timestamp ORDER BY timestamp",
        Tables::CPUMonitoring.into_str()
    );
    let mut stmt = cnx.prepare_cached(&query)?;
    let mut result = stmt.query([])?;
    let mut summaries = Vec::new();
    while let Some(row) = result.next()? {
        let summary = DumpSummary {
            timestamp: row.get(0)?,
            threads: row.get(1)?,
            max_cpu: row.get(2)?,
            total_cpu: row.get(3)?,
        };
        summaries.push(summary);
    }
    Ok(summaries)
}

pub fn get_cpu_dump_threads(cnx: &Connection, timestamp: u64) -> Result<Vec<CPUThread>, Error> {
    let query = format!(
        "SELECT tid, name, state, cpu FROM {} WHERE timestamp = $1",
        Tables::CPUMonitoring.into_str()
    );
    let mut stmt = cnx.prepare_cached(&query)?;
    let mut result = stmt.query([timestamp])?;
    let mut threads = Vec::new();
    while let Some(row) = result.next()? {
        let thread = CPUThread {
            tid: row.get(0)?,
            name: row.get(1)?,
            state: row.get(2)?,
            cpu: row.get(3)?,
        };
        threads.push(thread);
    }

    Ok(threads)
}

pub fn get_cpu_series(cnx: &Connection, tid: u64) -> Result<Vec<CPUPoint>, Error> {
    let query = format!(
        "SELECT cpu, timestamp FROM {} WHERE tid = $1 ORDER BY timestamp",
        Tables::CPUMonitoring.into_str()
    );
    let mut stmt = cnx.prepare_cached(&query)?;
    let mut result = stmt.query([tid])?;
    let mut points = Vec::new();
    while let Some(row) = result.next()? {
        let point = CPUPoint {
            cpu: row.get(0)?,
            timestamp: row.get(1)?,
        };
        points.push(point);
    }
    Ok(points)
}
