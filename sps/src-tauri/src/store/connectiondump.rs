use std::u64;

use duckdb::Connection;

use crate::{
    handlers::types::ConnectionDumpSignal,
    store::{self, error::Error, tables::Tables},
};

pub fn get_connectiondump_signals(
    cnx: &Connection,
    from: Option<u64>,
    to: Option<u64>,
) -> Result<Vec<ConnectionDumpSignal>, Error> {
    let query = format!(
        "SELECT timestamp, tid, cause, suppressed FROM {0} WHERE timestamp BETWEEN $1 AND $2 AND cause IS NOT NULL ORDER BY timestamp",
        Tables::ConnectionDump.into_str(),
    );

    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([
        from.unwrap_or(u64::MIN),
        to.unwrap_or(u64::MAX)
    ])?;
    let mut result = Vec::new();
    while let Some(row) = rows.next()? {
        result.push(ConnectionDumpSignal {
            timestamp: row.get(0)?,
            tid: row.get(1)?,
            cause: row.get(2)?,
            suppressed: row.get(3)?,
        });
    }
    Ok(result)
}
