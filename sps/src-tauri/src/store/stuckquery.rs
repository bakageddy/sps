use std::str::FromStr;

use duckdb::Connection;

use crate::handlers::types::{BlockingSnapshot, MSSQLSnapshot, PGSQLSnapshot};
use crate::parser::stuckquery::{PGSQLQuery, PGSQLState};
use crate::store::error::Error;
use crate::store::tables::Tables;

pub fn get_stuckquery_mssql_snapshots(cnx: &Connection) -> Result<Vec<MSSQLSnapshot>, Error> {
    let query = format!(
        "SELECT timestamp, COUNT(session_id), COUNT(blocked_by) FILTER (WHERE blocked_by != 0) FROM {0} GROUP BY timestamp ORDER BY timestamp",
        Tables::StuckqueryMSSQL.into_str()
    );
    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([])?;
    let mut snapshots = Vec::new();
    while let Some(row) = rows.next()? {
        let snapshot = MSSQLSnapshot {
            timestamp: row.get(0)?,
            queries: row.get(1)?,
            blocked: row.get(2)?,
        };
        snapshots.push(snapshot);
    }
    Ok(snapshots)
}

pub fn get_stuckquery_mssql_blocking_snapshots(
    cnx: &Connection,
) -> Result<Vec<BlockingSnapshot>, Error> {
    let query = format!(
        "SELECT timestamp, COUNT(DISTINCT head_blocker), COUNT(session_id) FROM {0} GROUP BY timestamp ORDER BY timestamp",
        Tables::StuckqueryBlockingMSSQL.into_str()
    );
    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([])?;
    let mut snapshots = Vec::new();
    while let Some(row) = rows.next()? {
        let snapshot = BlockingSnapshot {
            timestamp: row.get(0)?,
            chains: row.get(1)?,
            sessions: row.get(2)?,
        };
        snapshots.push(snapshot);
    }
    Ok(snapshots)
}

pub fn get_stuckquery_pgsql_snapshots(cnx: &Connection) -> Result<Vec<PGSQLSnapshot>, Error> {
    let query = format!(
        "SELECT timestamp, COUNT(pid), COUNT(waiting) FILTER (WHERE waiting = true), COUNT(state) FILTER (WHERE state = 'idle in transaction') FROM {0} GROUP BY timestamp ORDER BY timestamp",
        Tables::StuckqueryPGSQL.into_str()
    );
    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([])?;
    let mut snapshots = Vec::new();
    while let Some(row) = rows.next()? {
        let snapshot = PGSQLSnapshot {
            timestamp: row.get(0)?,
            queries: row.get(1)?,
            waiting: row.get(2)?,
            idle_in_txn: row.get(3)?,
        };
        snapshots.push(snapshot);
    }
    Ok(snapshots)
}

pub fn get_stuckquery_pgsql_queries<'a, 'b>(
    cnx: &'a Connection,
    timestamp: u64,
) -> Result<Vec<PGSQLQuery<'b>>, Error> {
    let query = format!(
        "SELECT pid, query_time, txn_time, db_name, state, waiting, query, state_change, application_name, client_host, client_port FROM {} WHERE timestamp = $1",
        Tables::StuckqueryPGSQL.into_str()
    );
    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([timestamp])?;
    let mut queries = Vec::new();
    while let Some(row) = rows.next()? {
        let state: String = row.get(4)?;
        let state = PGSQLState::from_str(&state).unwrap_or(PGSQLState::Active);
        let query = PGSQLQuery {
            pid: row.get(0)?,
            query_time: row.get(1)?,
            txn_time: row.get(2)?,
            db_name: row.get::<usize, String>(3)?.into(),
            state,
            waiting: row.get(5)?,
            query: row.get::<usize, String>(6)?.into(),
            state_change: row.get(7)?,
            application_name: row.get::<usize, Option<String>>(8)?.map(|a| a.into()),
            client_addr: None,
            client_host: row.get::<usize, Option<String>>(9)?.map(|h| h.into()),
            client_port: row.get::<usize, Option<u16>>(10)?,
        };
        queries.push(query);
    }
    Ok(queries)
}
