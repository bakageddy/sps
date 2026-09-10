use std::str::FromStr;

use duckdb::Connection;
use duckdb::types::Value;

use crate::handlers::types::{
    BlockingSnapshot, MSSQLLongRunningQuery, MSSQLLongRunningTxn, MSSQLSnapshot,
    PGSQLLongRunningQuery, PGSQLSnapshot,
};
use crate::parser::WaitType;
use crate::parser::stuckquery::{BlockingQuery, MSSQLStatus, PGSQLQuery, PGSQLState, RunningQuery};
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

pub fn get_stuckquery_mssql_queries<'a>(
    cnx: &Connection,
    timestamp: u64,
) -> Result<Vec<RunningQuery<'a>>, Error> {
    let query = format!(
        "SELECT session_id, status, txn_id, blocked_by, wait_type, wait_resource, wait_time_ms, cpu_time_ms, logical_reads, reads, writes, elapsed, statement, command_text, command, login, host, db, program, host_process, last_request_end, login_time, open_txn FROM {0} WHERE {0}.timestamp = $1",
        Tables::StuckqueryMSSQL.into_str()
    );
    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([timestamp])?;
    let mut queries = Vec::new();
    while let Some(row) = rows.next()? {
        let status =
            MSSQLStatus::from_str(&row.get::<usize, String>(1)?).unwrap_or(MSSQLStatus::Background);
        let wait_type = row
            .get::<usize, Option<String>>(4)?
            .map(|s| WaitType::parse(&s));
        let query = RunningQuery {
            session_id: row.get(0)?,
            status,
            txn_id: row.get(2)?,
            blocked_by: row.get(3)?,
            wait_type,
            wait_resource: row.get::<usize, Option<String>>(5)?.map(|s| s.into()),
            wait_time_ms: row.get(6)?,
            cpu_time_ms: row.get(7)?,
            logical_reads: row.get(8)?,
            reads: row.get(9)?,
            writes: row.get(10)?,
            elapsed: row.get(11)?,
            statement: row.get::<usize, String>(12)?.into(),
            command_text: row.get::<usize, String>(13)?.into(),
            command: row.get::<usize, String>(14)?.into(),
            login: row.get::<usize, String>(15)?.into(),
            host: row.get::<usize, String>(16)?.into(),
            db: row.get::<usize, String>(17)?.into(),
            program: row.get::<usize, String>(18)?.into(),
            host_process: row.get(19)?,
            last_request_end: row.get(20)?,
            login_time: row.get(21)?,
            open_txn: row.get(22)?,
        };
        queries.push(query);
    }
    Ok(queries)
}

pub fn get_stuckquery_mssql_blocking<'a>(
    cnx: &Connection,
    timestamp: u64,
) -> Result<Vec<BlockingQuery<'a>>, Error> {
    let query = format!(
        "SELECT head_blocker, session_id, txn_id, blocking_session_id, wait_type, wait_duration, wait_resource, statement_start_offset, statement_end_offset, plan_handle, sql_handle, most_recent_sql_handle, level, blocker_query_or_most_recent_query FROM {0} WHERE {0}.timestamp = $1",
        Tables::StuckqueryBlockingMSSQL.into_str()
    );

    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([timestamp])?;
    let mut queries = Vec::new();
    while let Some(row) = rows.next()? {
        let wait_type = row
            .get::<usize, Option<String>>(4)?
            .map(|s| WaitType::parse(&s));
        let query = BlockingQuery {
            head_blocker: row.get(0)?,
            session_id: row.get(1)?,
            txn_id: row.get(2)?,
            blocking_session_id: row.get(3)?,
            wait_type,
            wait_duration: row.get(5)?,
            wait_resource: row.get::<usize, Option<String>>(6)?.map(|s| s.into()),
            statement_start_offset: row.get(7)?,
            statement_end_offset: row.get(8)?,
            plan_handle: row.get::<usize, String>(9)?.into(),
            sql_handle: row.get::<usize, String>(10)?.into(),
            most_recent_sql_handle: row.get::<usize, String>(11)?.into(),
            level: row.get(12)?,
            blocker_query_or_most_recent_query: row.get::<usize, String>(13)?.into(),
        };
        queries.push(query);
    }
    Ok(queries)
}

pub fn get_stuckquery_mssql_long_running(
    cnx: &Connection,
) -> Result<Vec<MSSQLLongRunningQuery>, Error> {
    let query = format!(
        "SELECT session_id, txn_id, statement, login, COUNT(timestamp), MIN(timestamp), MAX(timestamp), MAX(elapsed), MAX(cpu_time_ms), COUNT(blocked_by) FILTER (WHERE blocked_by != 0) FROM {0} GROUP BY session_id, txn_id, statement, login HAVING COUNT(timestamp) > 1 ORDER BY timestamp",
        Tables::StuckqueryMSSQL.into_str()
    );
    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([])?;
    let mut queries = Vec::new();
    while let Some(row) = rows.next()? {
        let query = MSSQLLongRunningQuery {
            session_id: row.get(0)?,
            txn_id: row.get(1)?,
            statement: row.get(2)?,
            login: row.get(3)?,
            snapshots: row.get(4)?,
            first_seen: row.get(5)?,
            last_seen: row.get(6)?,
            max_elapsed: row.get(7)?,
            max_cpu_time_ms: row.get(8)?,
            blocked_in: row.get(9)?,
        };
        queries.push(query);
    }

    Ok(queries)
}

pub fn get_stuckquery_pgsql_long_running(
    cnx: &Connection,
) -> Result<Vec<PGSQLLongRunningQuery>, Error> {
    let query = format!(
        "SELECT pid, query, FIRST(db_name), COUNT(timestamp), MIN(timestamp), MAX(timestamp), MAX(query_time), COUNT(state) FILTER (WHERE state = 'idle in transaction') FROM {} GROUP BY pid, query HAVING COUNT(timestamp) > 1 ORDER BY timestamp",
        Tables::StuckqueryPGSQL.into_str()
    );
    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([])?;
    let mut queries = Vec::new();
    while let Some(row) = rows.next()? {
        let query = PGSQLLongRunningQuery {
            pid: row.get(0)?,
            query: row.get(1)?,
            db_name: row.get(2)?,
            snapshots: row.get(3)?,
            first_seen: row.get(4)?,
            last_seen: row.get(5)?,
            max_query_time_ms: row.get(6)?,
            idle_in_txn_in: row.get(7)?,
        };
        queries.push(query);
    }
    Ok(queries)
}

pub fn get_stuckquery_mssql_long_running_txn(
    cnx: &Connection,
) -> Result<Vec<MSSQLLongRunningTxn>, Error> {
    let query = format!(
        "SELECT session_id, txn_id, FIRST(login), COUNT(timestamps), MIN(timestamps), MAX(timestamps), LIST(statement_text) FROM {} GROUP BY session_id, txn_id ORDER BY timestamp",
        Tables::StuckqueryMSSQL.into_str()
    );

    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([])?;
    let mut result = Vec::new();
    while let Some(row) = rows.next()? {
        let queries: Value = row.get(6)?;
        let queries: Vec<String> = match queries {
            Value::List(items) => items
                .into_iter()
                .flat_map(|v| match v {
                    Value::Text(s) => Ok(s),
                    _ => Err(()),
                })
                .collect(),
            _ => Vec::new(),
        };
        let query = MSSQLLongRunningTxn {
            session_id: row.get(0)?,
            txn_id: row.get(1)?,
            login: row.get(2)?,
            snapshots: row.get(3)?,
            first_seen: row.get(4)?,
            last_seen: row.get(5)?,
            queries,
        };
        result.push(query);
    }
    Ok(result)
}
