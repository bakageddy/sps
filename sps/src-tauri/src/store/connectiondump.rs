use std::u64;

use duckdb::{Connection, OptionalExt};

use crate::{
    handlers::types::ConnectionDumpSnapshot,
    parser::connectiondump::{Signal, Stats},
    store::{error::Error, tables::Tables},
};

pub fn get_connectiondump_signals(
    cnx: &Connection,
    from: Option<u64>,
    to: Option<u64>,
) -> Result<Vec<Signal>, Error> {
    let query = format!(
        "SELECT timestamp, tid, cause, suppressed FROM {0} WHERE timestamp BETWEEN $1 AND $2 AND cause IS NOT NULL ORDER BY timestamp",
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
        "SELECT timestamp, tid, COUNT(id), MAX(duration) FROM {0} GROUP BY timestamp, tid ORDER BY timestamp",
        Tables::ConnectionDumpTraces
    );
    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([])?;
    let mut results = Vec::new();
    let stats_query = format!(
        "SELECT timestamp, tid, used, free, total FROM {0} WHERE timestamp = $1 AND tid = $2",
        Tables::ConnectionDump
    );
    let mut stats_stmt = cnx.prepare_cached(&stats_query)?;
    while let Some(row) = rows.next()? {
        let timestamp: u64 = row.get(0)?;
        let tid: u64 = row.get(1)?;
        let result = stats_stmt
            .query_one([timestamp, tid], |r| -> Result<(u64, u64), duckdb::Error> {
                Ok((r.get(3)?, r.get(4)?))
            });
        match result {
            Ok((used, free)) => (used, free),
            Err(duckdb::Error::QueryReturnedNoRows) => 
        }

        results.push();
    }

    todo!()
}
