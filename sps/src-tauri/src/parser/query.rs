use error::Error;
use error::MSSQLStatusParse;
use error::PGSQLStateParse;
use serde::Deserialize;
use serde::Serialize;
use std::borrow::Cow;
use std::net::Ipv4Addr;
use std::str::FromStr;
use time::OffsetDateTime;

use crate::parser::WaitType;
use crate::parser::query::error::SPWho2StatusParse;
use crate::parser::tokenizer::Parser;
use crate::parser::tokenizer::Tokenizer;
use crate::util;
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;

const PGSQL_QUERY_STATE_CHANGE: &[BorrowedFormatItem] = format_description!(
    "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond][offset_hour sign:mandatory]:[offset_minute]"
);

const MSSQL_LOGIN_TIME_FORMAT: &[BorrowedFormatItem] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]");

const SPWHO2_LAST_BATCH_FORMAT: &[BorrowedFormatItem] =
    format_description!("[year]/[month]/[day] [hour]:[minute]:[second]");

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PGSQLQuery<'a> {
    pub pid: u64,
    pub query_time: Option<u64>,
    pub txn_time: Option<u64>,
    pub db_name: Cow<'a, str>,
    pub state: PGSQLState,
    pub waiting: bool,
    pub query: Cow<'a, str>,
    pub state_change: u64,
    pub application_name: Option<Cow<'a, str>>,
    pub client_addr: Option<u32>,
    pub client_host: Option<Cow<'a, str>>,
    pub client_port: Option<u16>,
}

impl<'a> Parser<'a> for PGSQLQuery<'a> {
    type Error = Error;

    fn parse(data: &'a str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut tok = Tokenizer::new(data);
        let pid = tok.take_within_exclusive("|", "|")?.trim().parse()?;

        let query_time = tok.take_within_exclusive("|", "|")?.trim();
        let query_time = if query_time.is_empty() {
            None
        } else {
            let query_time = query_time.parse::<f32>()?;
            let query_time = (query_time * 1000.0f32).trunc() as u64;
            Some(query_time)
        };

        let txn_time = tok.take_within_exclusive("|", "|")?.trim();
        let txn_time = if txn_time.is_empty() {
            None
        } else {
            let txn_time = txn_time.parse::<f32>()?;
            let txn_time = (txn_time * 1000.0f32).trunc() as u64;
            Some(txn_time)
        };

        let db_name = tok.take_within_exclusive("|", "|")?.trim().into();
        let state = tok.take_within_exclusive("|", "|")?.trim().parse()?;

        let waiting = tok.take_within_exclusive("|", "|")?.trim() == "t";
        let query = tok.take_within_exclusive("|", "  |  ")?.trim().into();
        let state_change = tok.take_within_exclusive("|", "|")?.trim();
        let state_change = util::utc_unix_timestamp_millis(state_change, PGSQL_QUERY_STATE_CHANGE)?;
        let application_name = tok.take_within_exclusive("|", "|")?.trim();
        let application_name = if application_name.is_empty() {
            None
        } else {
            Some(application_name.into())
        };

        let client_addr = tok.take_within_exclusive("|", "|")?.trim();
        let client_addr = if client_addr.is_empty() {
            None
        } else {
            Some(client_addr.parse::<Ipv4Addr>()?.to_bits())
        };

        let client_host = tok.take_within_exclusive("|", "|")?.trim();
        let client_host = if client_host.is_empty() {
            None
        } else {
            Some(client_host.into())
        };
        let client_port = tok.take_within_exclusive("|", "|")?.trim();
        let client_port = if client_port.is_empty() {
            None
        } else {
            let client_port = client_port.parse()?;
            Some(client_port)
        };

        Ok(Self {
            pid,
            query_time,
            txn_time,
            db_name,
            state,
            waiting,
            query,
            state_change,
            application_name,
            client_addr,
            client_host,
            client_port,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PGSQLState {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "idle in transaction")]
    Idle,
}

impl FromStr for PGSQLState {
    type Err = PGSQLStateParse;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(Self::Active),
            "idle in transaction" => Ok(Self::Idle),
            _ => Err(PGSQLStateParse::UnknownState(s.to_owned())),
        }
    }
}

impl PGSQLState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Idle => "idle in transaction",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MSSQLQuery<'a> {
    pub session_id: u64,
    pub status: MSSQLStatus,
    pub txn_id: u64,
    pub blocked_by: u64,
    pub wait_type: Option<WaitType>,
    pub wait_resource: Option<Cow<'a, str>>,
    pub wait_time_ms: u64,
    pub cpu_time_ms: u64,
    pub logical_reads: u64,
    pub reads: u64,
    pub writes: u64,
    pub elapsed: u64,
    pub statement: Cow<'a, str>,
    pub command_text: Cow<'a, str>,
    pub command: Cow<'a, str>,
    pub login: Cow<'a, str>,
    pub host: Cow<'a, str>,
    pub db: Cow<'a, str>,
    pub program: Cow<'a, str>,
    pub host_process: u64,
    pub last_request_end: u64,
    pub login_time: u64,
    pub open_txn: u64,
}

impl<'a> Parser<'a> for MSSQLQuery<'a> {
    type Error = Error;

    fn parse(data: &'a str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut tok = Tokenizer::new(data);
        let session_id = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        let status = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        let txn_id = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        let blocked_by = tok.take_within_exclusive("|", "|")?.trim().parse()?;

        let wait_type = tok.take_within_exclusive("|", "|")?.trim();
        let wait_type = if wait_type.is_empty() {
            None
        } else {
            Some(WaitType::parse(wait_type))
        };

        let wait_resource = tok.take_within_exclusive("|", "|")?.trim();
        let wait_resource = if wait_resource.is_empty() {
            None
        } else {
            Some(wait_resource.into())
        };

        let wait_time_ms: f32 = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        let wait_time_ms = (wait_time_ms * 1000.0f32).trunc() as u64;

        let cpu_time_ms: f32 = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        let cpu_time_ms = (cpu_time_ms * 1000.0f32).trunc() as u64;
        let logical_reads = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        let reads = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        let writes = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        let elapsed: f32 = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        let elapsed = (elapsed * 1000.0f32).trunc() as u64;

        let statement = tok.take_within_exclusive("|", "  |  ")?.trim().into();
        let command_text = tok.take_within_exclusive("|", "|")?.trim().into();
        let command = tok.take_within_exclusive("|", "|")?.trim().into();
        let login = tok.take_within_exclusive("|", "|")?.trim().into();
        let host = tok.take_within_exclusive("|", "|")?.trim().into();
        let db = tok.take_within_exclusive("|", "|")?.trim().into();
        let program = tok.take_within_exclusive("|", "|")?.trim().into();
        let host_process = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        let last_request_end = tok.take_within_exclusive("|", "|")?.trim();
        let last_request_end =
            util::utc_unix_timestamp_millis(last_request_end, MSSQL_LOGIN_TIME_FORMAT)?;
        let login_time = tok.take_within_exclusive("|", "|")?.trim();
        let login_time = util::utc_unix_timestamp_millis(login_time, MSSQL_LOGIN_TIME_FORMAT)?;
        let open_txn = tok.take_within_exclusive("|", "|")?.trim().parse()?;

        Ok(Self {
            session_id,
            status,
            txn_id,
            blocked_by,
            wait_type,
            wait_resource,
            wait_time_ms,
            cpu_time_ms,
            logical_reads,
            reads,
            writes,
            elapsed,
            statement,
            command_text,
            command,
            login,
            host,
            db,
            program,
            host_process,
            last_request_end,
            login_time,
            open_txn,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MSSQLStatus {
    Background,
    Rollback,
    Running,
    Runnable,
    Sleeping,
    Suspended,
}

impl FromStr for MSSQLStatus {
    type Err = MSSQLStatusParse;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "running" => Ok(Self::Running),
            "runnable" => Ok(Self::Runnable),
            "rollback" => Ok(Self::Rollback),
            "sleeping" => Ok(Self::Sleeping),
            "background" => Ok(Self::Background),
            "suspended" => Ok(Self::Suspended),
            _ => Err(MSSQLStatusParse::UnknownStatus(s.to_owned())),
        }
    }
}

impl MSSQLStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MSSQLStatus::Background => "background",
            MSSQLStatus::Rollback => "rollback",
            MSSQLStatus::Running => "running",
            MSSQLStatus::Runnable => "runnable",
            MSSQLStatus::Sleeping => "sleeping",
            MSSQLStatus::Suspended => "suspended",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockingQuery<'a> {
    pub head_blocker: u64,
    pub session_id: u64,
    pub txn_id: u64,
    pub blocking_session_id: u64,
    pub wait_type: Option<WaitType>,
    pub wait_duration: Option<u64>,
    pub wait_resource: Option<Cow<'a, str>>,
    pub statement_start_offset: i64,
    pub statement_end_offset: i64,
    pub plan_handle: Cow<'a, str>,
    pub sql_handle: Cow<'a, str>,
    pub most_recent_sql_handle: Cow<'a, str>,
    pub level: u64,
    pub blocker_query_or_most_recent_query: Cow<'a, str>,
}

impl<'a> Parser<'a> for BlockingQuery<'a> {
    type Error = Error;

    fn parse(data: &'a str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut tok = Tokenizer::new(data);
        let head_blocker = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        let session_id = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        let txn_id = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        let blocking_session_id = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        let wait_type = tok.take_within_exclusive("|", "|")?.trim();
        let wait_type = if wait_type.is_empty() {
            None
        } else {
            Some(WaitType::parse(wait_type))
        };
        let wait_duration = tok.take_within_exclusive("|", "|")?.trim();
        let wait_duration = if wait_duration.is_empty() {
            None
        } else {
            Some(wait_duration.parse()?)
        };
        let wait_resource = tok.take_within_exclusive("|", "|")?.trim();
        let wait_resource = if wait_resource.is_empty() {
            None
        } else {
            Some(wait_resource.into())
        };
        let statement_start_offset = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        let statement_end_offset = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        let plan_handle = tok.take_within_exclusive("|", "|")?.trim().into();
        let sql_handle = tok.take_within_exclusive("|", "|")?.trim().into();
        let most_recent_sql_handle = tok.take_within_exclusive("|", "|")?.trim().into();
        let level = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        let blocker_query_or_most_recent_query = tok.take_within_exclusive("|", "|")?.trim().into();

        Ok(Self {
            head_blocker,
            session_id,
            txn_id,
            blocking_session_id,
            wait_type,
            wait_duration,
            wait_resource,
            statement_start_offset,
            statement_end_offset,
            plan_handle,
            sql_handle,
            most_recent_sql_handle,
            level,
            blocker_query_or_most_recent_query,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SPWho2Query<'a> {
    pub spid: u64,
    pub status: SPWho2Status,
    pub login: Cow<'a, str>,
    pub hostname: Option<Cow<'a, str>>,
    pub blocked_by: Option<u64>,
    pub dbname: Cow<'a, str>,
    pub command: Cow<'a, str>,
    pub cputime: u64,
    pub diskio: u64,
    pub lastbatch: u64,
    pub program_name: Option<Cow<'a, str>>,
    pub request_id: u64,
}

impl<'a> Parser<'a> for SPWho2Query<'a> {
    type Error = Error;

    fn parse(data: &'a str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut tok = Tokenizer::new(data);
        let spid = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        let status = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        let login = tok.take_within_exclusive("|", "|")?.trim().into();
        let hostname = tok.take_within_exclusive("|", "|")?.trim();
        let hostname = if hostname.is_empty() || hostname == "." {
            None
        } else {
            Some(hostname.into())
        };
        let blocked_by = tok.take_within_exclusive("|", "|")?.trim();
        let blocked_by = if blocked_by.is_empty() || blocked_by == "." {
            None
        } else {
            Some(blocked_by.parse()?)
        };
        let dbname = tok.take_within_exclusive("|", "|")?.trim().into();
        let command = tok.take_within_exclusive("|", "|")?.trim().into();
        let cputime = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        let diskio = tok.take_within_exclusive("|", "|")?.trim().parse()?;
        let lastbatch = tok.take_within_exclusive("|", "|")?.trim();
        let lastbatch = format!("{}/{lastbatch}", OffsetDateTime::now_utc().year());
        let lastbatch = util::utc_unix_timestamp_millis(&lastbatch, SPWHO2_LAST_BATCH_FORMAT)?;
        let program_name = tok.take_within_exclusive("|", "|")?.trim();
        let program_name = if program_name.is_empty() {
            None
        } else {
            Some(program_name.into())
        };
        // NOTE: spid is repeated twice in this table
        let _ = tok.take_within_exclusive("|", "|")?;
        let request_id = tok.take_within_exclusive("|", "|")?.trim().parse()?;

        Ok(Self {
            spid,
            status,
            login,
            hostname,
            blocked_by,
            dbname,
            command,
            cputime,
            diskio,
            lastbatch,
            program_name,
            request_id,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SPWho2Status {
    #[serde(rename = "sleeping")]
    Sleeping,
    #[serde(rename = "BACKGROUND")]
    Background,
    #[serde(rename = "RUNNABLE")]
    Runnable,
    #[serde(rename = "SUSPENDED")]
    Suspended,
    #[serde(rename = "DORMANT")]
    Dormant,
}

impl FromStr for SPWho2Status {
    type Err = SPWho2StatusParse;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sleeping" => Ok(Self::Sleeping),
            "BACKGROUND" => Ok(Self::Background),
            "RUNNABLE" => Ok(Self::Runnable),
            "SUSPENDED" => Ok(Self::Suspended),
            "DORMANT" => Ok(Self::Dormant),
            _ => Err(SPWho2StatusParse::UnknownStatus(s.to_owned())),
        }
    }
}

impl SPWho2Status {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dormant => "DORMANT",
            Self::Background => "BACKGROUND",
            Self::Runnable => "RUNNABLE",
            Self::Sleeping => "sleeping",
            Self::Suspended => "SUSPENDED",
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::BlockingQuery;
    use super::MSSQLQuery;
    use super::MSSQLStatus;
    use super::PGSQLQuery;
    use super::PGSQLState;
    use crate::parser::TableKind;
    use crate::parser::WaitType;
    use crate::parser::tokenizer::Parser;
    use crate::parser::tokenizer::Tokenizer;
    use crate::util;
    use std::assert_matches;
    use std::ops::Deref;

    #[test]
    fn pgsql_query_single_line() {
        let line = "|  39704  |  1474.170226     |  1474.170226    |  servicedesk  |  active               |  f        |  autovacuum: VACUUM pg_toast.pg_toast_1153741                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |  2026-09-08 00:59:32.124624+05:30  |                          |                  |                   |               |";
        let result = PGSQLQuery::parse(line);
        assert!(
            result.is_ok(),
            "Error during parsing: {}",
            result.unwrap_err()
        );
        let result = result.unwrap();
        assert_eq!(39704, result.pid);
        assert_eq!(Some(1474170), result.query_time);
        assert_eq!(Some(1474170), result.txn_time);
        assert_eq!("servicedesk", result.db_name);
        assert_matches!(result.state, PGSQLState::Active);
        assert_eq!(false, result.waiting);
        assert_eq!(None, result.application_name);
        assert_eq!(None, result.client_addr);
        assert_eq!(None, result.client_host);
        assert_eq!(None, result.client_port);
    }

    #[test]
    fn mssql_query_single_line() {
        let query = r"|  114         |  suspended  |  35839033162  |  0           |  PAGEIOLATCH_EX  |  5:1:134959573  |  0.000000       |  402.755000    |  144979671      |  5529498   |  2510814  |  1319.037000       |  DBCC SHRINKDATABASE(N'sdpload15140' )                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |                |  DbccFilesCompact  |  SDP-DB-W221\Administrator  |  SDP-DB-W221  |  sdpload15140  |  SQL Server Management Studio          |  24968            |  2026-09-07 23:15:39.013  |  2026-09-07 23:15:25.7    |  1                       |";
        let result = MSSQLQuery::parse(query);
        assert!(
            result.is_ok(),
            "Error during parsing: {}",
            result.unwrap_err()
        );
        let result = result.unwrap();
        assert_eq!(result.session_id, 114);
        assert_matches!(result.status, MSSQLStatus::Suspended);
        assert_eq!(result.txn_id, 35839033162);
        assert_eq!(result.blocked_by, 0);
        assert_matches!(result.wait_type, Some(WaitType::PAGEIOLATCH_EX));
        assert_eq!(result.wait_resource, Some("5:1:134959573".into()));
        assert_eq!(result.wait_time_ms, 0);
        assert_eq!(result.cpu_time_ms, 402755);
        assert_eq!(result.logical_reads, 144979671);
        assert_eq!(result.reads, 5529498);
    }

    #[test]
    fn mssql_blocking_query_single_line() {
        let map =
            util::map_file("test/stuckqueries/stuckquery_mssql_blocking_query_single_line.txt")
                .unwrap();
        let line = std::str::from_utf8(map.deref()).unwrap();
        let result = BlockingQuery::parse(line);
        assert!(
            result.is_ok(),
            "Error during parsing: {}",
            result.unwrap_err()
        );
        let result = result.unwrap();
        assert_eq!(result.head_blocker, 98);
        assert_eq!(result.session_id, 98);
        assert_eq!(result.txn_id, 35870378461);
        assert_eq!(result.blocking_session_id, 0);
        assert_eq!(result.wait_type, None);
        assert_eq!(result.wait_duration, Some(0));
        assert_eq!(result.wait_resource, None);
        assert_eq!(result.statement_start_offset, 0);
        assert_eq!(result.statement_end_offset, 2796);
        assert_eq!(result.level, 0);
    }

    #[test]
    fn stuckquery_mssql_single_blocking_query_table() {
        let map =
            util::map_file("test/stuckqueries/stuckquery_mssql_blocking_query_single_table.txt")
                .unwrap();
        let mut tok = Tokenizer::from_bytes(map.deref()).unwrap();
        let mut header = Vec::new();
        for _ in 0..5 {
            header.push(tok.get_line().unwrap());
        }

        let table_kind = TableKind::detect_table(&header);
        assert_matches!(table_kind, Some(TableKind::MSSQLBlockingQuery));
        let mut count = 0;
        while let Some(line) = tok.get_line() {
            let result = BlockingQuery::parse(line);
            assert!(
                result.is_ok(),
                "Error during parsing: {}",
                result.unwrap_err()
            );
            count += 1;
        }
        assert_eq!(count, 35);
    }
}

pub mod error {
    use std::{
        net::AddrParseError,
        num::{ParseFloatError, ParseIntError},
    };

    use crate::parser::tokenizer;
    use crate::types::TimestampError;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Invalid Format in parsing: {0}")]
        InvalidFormat(#[from] tokenizer::error::Error),

        #[error("Parsing Integer: {0}")]
        Integer(#[from] ParseIntError),

        #[error("Parsing Floating point number: {0}")]
        Float(#[from] ParseFloatError),

        #[error("Parsing IP address: {0}")]
        IpAddr(#[from] AddrParseError),

        #[error("Parsing timestamps: {0}")]
        Time(#[from] TimestampError),

        #[error("Parsing PGSQL State: {0}")]
        PGState(#[from] PGSQLStateParse),

        #[error("Parsing MSSQL Status: {0}")]
        MSSQLStatus(#[from] MSSQLStatusParse),

        #[error("Parsing MSSQL SPWho2Status: {0}")]
        SPWho2Status(#[from] SPWho2StatusParse),
    }

    #[derive(Debug, thiserror::Error)]
    pub enum PGSQLStateParse {
        #[error("Unknown PGSQL Query State: {0}")]
        UnknownState(String),
    }

    #[derive(Debug, thiserror::Error)]
    pub enum MSSQLStatusParse {
        #[error("Unknown MSSQL Query Status: {0}")]
        UnknownStatus(String),
    }

    #[derive(Debug, thiserror::Error)]
    pub enum SPWho2StatusParse {
        #[error("Unknown SPWho2 Query Status: {0}")]
        UnknownStatus(String),
    }
}
