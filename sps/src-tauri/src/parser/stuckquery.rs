use std::{
    borrow::Cow,
    collections::HashSet,
    net::Ipv4Addr,
    str::{FromStr, Utf8Error},
};

use crate::{
    parser::{
        DBKind, WaitType,
        stuckquery::error::{ColumnDataError, MSSQLStatusParse, PGSQLStateParse},
        tokenizer::{self, Parser, Tokenizer},
    },
    util,
};
use error::Error;
use time::{format_description::BorrowedFormatItem, macros::format_description};
use tracing::warn;

const STUCKQUERY_TIME_FORMAT: &[BorrowedFormatItem] =
    format_description!("[hour]:[minute]:[second].[subsecond]");
const STUCKQUERY_DATE_FORMAT: &[BorrowedFormatItem] = format_description!("[day]-[month]-[year]");
const STUCKQUERY_STATE_CHANGE_FORMAT: &[BorrowedFormatItem] = format_description!(
    "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond][offset_hour sign:mandatory]:[offset_minute]"
);
const STUCKQUERY_LOGIN_TIME_FORMAT: &[BorrowedFormatItem] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]");

#[derive(Debug)]
pub struct StuckqueryParser<'a>(&'a str, ParserState);
impl<'a> StuckqueryParser<'a> {
    // const STUCKQUERY_PGSQL_RUNNING_QUERY_HEADER: &'static str = "Currently Running Queries";
    const STUCKQUERY_MSSQL_RUNNING_QUERY_HEADER: &'static str = "Currently Running Queries";
    const STUCKQUERY_MSSQL_BLOCKING_QUERY_HEADER: &'static str = "Currently Blocking Query Details";
    pub fn new(data: &'a str) -> Self {
        Self(data, ParserState::Initial)
    }

    pub fn parse_header(header: &str) -> Result<u64, Error> {
        let mut htok = Tokenizer::new(header);
        let time = htok.take_within("[", "]")?;
        let date = htok.take_within("[", "]")?;
        let timestamp = util::unix_timestamp_millis(
            time,
            date,
            STUCKQUERY_TIME_FORMAT,
            STUCKQUERY_DATE_FORMAT,
        )?;
        Ok(timestamp)
    }

    pub fn detect_kind(table_header_lines: &[&str]) -> Option<DBKind> {
        let table_column_names = table_header_lines.get(table_header_lines.len() - 2)?;
        let mut tok = Tokenizer::new(*table_column_names);
        let mut columns = HashSet::new();
        while let Ok(column_name) = tok.take_within_exclusive("|", "|") {
            columns.insert(column_name.trim());
        }

        if columns.contains("pid") {
            Some(DBKind::PGSQL)
        } else if columns.contains("Session ID") || columns.contains("Logical Reads") {
            Some(DBKind::MSSQL)
        } else {
            None
        }
    }

    pub fn extract_table_name<'s, 'b>(table_header_lines: &'b [&'s str]) -> Option<&'s str> {
        let table_name = table_header_lines.get(1)?;
        let mut tok = Tokenizer::new(*table_name);
        tok.skip_whitespace();
        tok.take_within("|", "|").ok().map(|s| s.trim())
    }

    pub fn extract_mssql_blocking_queries(
        tok: &mut Tokenizer<'a>,
    ) -> Result<Vec<Stuckquery<'a>>, Error> {
        tok.skip_whitespace();
        let mut table_header_lines: [&str; 5] = [""; 5];
        let mut idx = 0;

        while let Some(line) = tok.peek_line()
            && line.trim_start().starts_with("|")
        {
            table_header_lines[idx] = line;
            idx += 1;
            let _ = tok.get_line();
            if idx == 5 {
                break;
            }
        }

        if idx != 5 {
            return Err(Error::InvalidTableHeader);
        }

        let table_name = Self::extract_table_name(&table_header_lines);
        if let Some(name) = table_name
            && name == Self::STUCKQUERY_MSSQL_BLOCKING_QUERY_HEADER
        {
            let mut queries = Vec::new();
            while let Some(line) = tok.peek_line()
                && line.trim_start().starts_with("|")
            {
                let query = BlockingQuery::parse(line)?;
                queries.push(Stuckquery::MSSQL(MSSQLQuery::Blocking(query)));
                let _ = tok.get_line();
            }
            Ok(queries)
        } else {
            return Err(Error::InvalidFormat(
                tokenizer::error::Error::DelimiterNotFound("|".to_owned()),
            ));
        }
    }
}

#[derive(Debug)]
enum ParserState {
    Initial,
    Header,
    QueryTable,
}

impl<'a> TryFrom<&'a [u8]> for StuckqueryParser<'a> {
    type Error = Utf8Error;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let data = std::str::from_utf8(value)?;
        Ok(Self::new(data))
    }
}

// BUG: Fix state.0 = tok.remaining()
impl<'a> Iterator for StuckqueryParser<'a> {
    type Item = Result<StuckqueryTable<'a>, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut tok = Tokenizer::new(self.0);
        tok.skip_whitespace();
        if tok.is_empty() {
            return None;
        }

        self.1 = ParserState::Initial;
        while let Some(line) = tok.peek_line()
            && line.trim_start().starts_with("[")
        {
            if line.trim_end().ends_with("::") {
                self.1 = ParserState::Header;
                break;
            }

            tok.get_line()?;
        }

        if !matches!(self.1, ParserState::Header) {
            self.0 = tok.remaining();
            return Some(Err(Error::UnableToFindTableHeader));
        }

        let header = tok.get_line()?;
        let timestamp = match Self::parse_header(header) {
            Ok(x) => x,
            Err(e) => return Some(Err(e)),
        };

        tok.skip_whitespace();

        let mut table_header_line_count = 0;
        let mut table_header_lines = Vec::new();
        while let Some(line) = tok.peek_line()
            && line.starts_with("|")
            && table_header_line_count != 5
        {
            table_header_lines.push(line);

            table_header_line_count += 1;
            let _ = tok.get_line()?;
        }

        if table_header_lines.len() != 5 {
            self.0 = tok.remaining();
            return Some(Err(Error::InvalidTableHeader));
        }

        let table_kind = Self::detect_kind(&table_header_lines);
        if table_kind.is_none() {
            self.0 = tok.remaining();
            return Some(Err(Error::UnableToDetectTableKind));
        }

        let table_name = Self::extract_table_name(&table_header_lines);
        if table_name.is_none() {
            self.0 = tok.remaining();
            return Some(Err(Error::UnableToDetectTableKind));
        }

        let table_kind = table_kind.unwrap();
        let table_name = table_name.unwrap();
        let mut queries = Vec::new();
        self.1 = ParserState::QueryTable;
        while let Some(line) = tok.peek_line()
            && line.trim_start().starts_with("|")
        {
            let query = match table_kind {
                DBKind::PGSQL => {
                    let query = PGSQLQuery::parse(line);
                    if query.is_err() {
                        return Some(Err(query.unwrap_err()));
                    }
                    Stuckquery::PGSQL(query.unwrap())
                }
                DBKind::MSSQL => {
                    // NOTE: I am not sure if i should refactor this code.
                    // Maybe I should
                    if table_name.eq(Self::STUCKQUERY_MSSQL_RUNNING_QUERY_HEADER) {
                        let query = RunningQuery::parse(line);
                        if query.is_err() {
                            return Some(Err(query.unwrap_err()));
                        }
                        Stuckquery::MSSQL(MSSQLQuery::Running(query.unwrap()))
                    } else if table_name.eq(Self::STUCKQUERY_MSSQL_BLOCKING_QUERY_HEADER) {
                        let query = BlockingQuery::parse(line);
                        if query.is_err() {
                            return Some(Err(query.unwrap_err()));
                        }
                        Stuckquery::MSSQL(MSSQLQuery::Blocking(query.unwrap()))
                    } else {
                        warn!("Unknown table name: {}", table_name);
                        continue;
                    }
                }
            };
            tok.get_line()?;
            queries.push(query);
        }

        if matches!(table_kind, DBKind::MSSQL) {
            let result = Self::extract_mssql_blocking_queries(&mut tok);
            if let Ok(table) = result {
                queries.extend(table);
            }
        }

        self.0 = tok.remaining();
        Some(Ok(StuckqueryTable { queries, timestamp }))
    }
}

#[derive(Debug)]
pub struct StuckqueryTable<'a> {
    pub queries: Vec<Stuckquery<'a>>,
    pub timestamp: u64,
}

#[derive(Debug)]
pub enum Stuckquery<'a> {
    PGSQL(PGSQLQuery<'a>),
    MSSQL(MSSQLQuery<'a>),
}

#[derive(Debug)]
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
        let pid = tok
            .take_within_exclusive("|", "|")?
            .trim()
            .parse()
            .map_err(|e| Error::Parse(String::from("pid"), error::ColumnDataError::Integer(e)))?;
        let query_time = tok.take_within_exclusive("|", "|")?.trim();
        let query_time = if query_time.is_empty() {
            None
        } else {
            let query_time = query_time.parse::<f32>().map_err(|e| {
                Error::Parse(
                    String::from("Query Time (s)"),
                    error::ColumnDataError::Float(e),
                )
            })?;
            let query_time = (query_time * 1000.0f32).trunc() as u64;
            Some(query_time)
        };
        let txn_time = tok.take_within_exclusive("|", "|")?.trim();
        let txn_time = if txn_time.is_empty() {
            None
        } else {
            let txn_time = txn_time.parse::<f32>().map_err(|e| {
                Error::Parse(
                    String::from("Txn Time (s)"),
                    error::ColumnDataError::Float(e),
                )
            })?;
            let txn_time = (txn_time * 1000.0f32).trunc() as u64;
            Some(txn_time)
        };
        let db_name = tok.take_within_exclusive("|", "|")?.trim().into();
        let state = tok
            .take_within_exclusive("|", "|")?
            .trim()
            .parse()
            .map_err(|e| Error::Parse(String::from("state"), ColumnDataError::PGSQLState(e)))?;
        let waiting = tok.take_within_exclusive("|", "|")?.trim() == "t";
        let query = tok.take_within_exclusive("|", "|")?.trim().into();
        let state_change = tok.take_within_exclusive("|", "|")?.trim();
        let state_change =
            util::utc_unix_timestamp_millis(state_change, STUCKQUERY_STATE_CHANGE_FORMAT)?;
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
            Some(
                client_addr
                    .parse::<Ipv4Addr>()
                    .map_err(|e| {
                        Error::Parse(String::from("Client Address"), ColumnDataError::IpV4Addr(e))
                    })?
                    .to_bits(),
            )
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
            let client_port = client_port.parse().map_err(|e| {
                Error::Parse(String::from("Client Port"), ColumnDataError::Integer(e))
            })?;
            Some(client_port)
        };

        Ok(PGSQLQuery {
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

#[derive(Debug)]
pub enum PGSQLState {
    Active,
    Idle,
}

impl FromStr for PGSQLState {
    type Err = PGSQLStateParse;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(Self::Active),
            "idle in transaction" => Ok(Self::Idle),
            _ => Err(PGSQLStateParse::UnknownState(String::from(s))),
        }
    }
}

impl PGSQLState {
    pub fn into(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Idle => "idle in transaction",
        }
    }
}

#[derive(Debug)]
pub enum MSSQLQuery<'a> {
    Blocking(BlockingQuery<'a>),
    Running(RunningQuery<'a>),
}

#[derive(Debug)]
pub struct RunningQuery<'a> {
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

#[derive(Debug)]
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
    pub fn into_str(&self) -> &'static str {
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

impl<'a> Parser<'a> for RunningQuery<'a> {
    type Error = Error;
    fn parse(data: &'a str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut tok = Tokenizer::new(data);
        let session_id = tok
            .take_within_exclusive("|", "|")?
            .trim()
            .parse()
            .map_err(|e| Error::Parse("Session ID".to_owned(), ColumnDataError::Integer(e)))?;
        let status = tok
            .take_within_exclusive("|", "|")?
            .trim()
            .parse()
            .map_err(|e| Error::Parse("Status".to_owned(), ColumnDataError::MSSQLStatus(e)))?;
        let txn_id = tok
            .take_within_exclusive("|", "|")?
            .trim()
            .parse()
            .map_err(|e| Error::Parse("Txn ID".to_owned(), ColumnDataError::Integer(e)))?;
        let blocked_by = tok
            .take_within_exclusive("|", "|")?
            .trim()
            .parse()
            .map_err(|e| Error::Parse("Blocked by".to_owned(), ColumnDataError::Integer(e)))?;

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

        let wait_time_ms: f32 = tok
            .take_within_exclusive("|", "|")?
            .trim()
            .parse()
            .map_err(|e| Error::Parse("Wait Time".to_owned(), ColumnDataError::Float(e)))?;
        let wait_time_ms = (wait_time_ms * 1000.0f32).trunc() as u64;

        let cpu_time_ms: f32 = tok
            .take_within_exclusive("|", "|")?
            .trim()
            .parse()
            .map_err(|e| Error::Parse("CPU Time".to_owned(), ColumnDataError::Float(e)))?;
        let cpu_time_ms = (cpu_time_ms * 1000.0f32).trunc() as u64;

        let logical_reads = tok
            .take_within_exclusive("|", "|")?
            .trim()
            .parse()
            .map_err(|e| Error::Parse("Logical Reads".to_owned(), ColumnDataError::Integer(e)))?;

        let reads = tok
            .take_within_exclusive("|", "|")?
            .trim()
            .parse()
            .map_err(|e| Error::Parse("Physical Reads".to_owned(), ColumnDataError::Integer(e)))?;

        let writes = tok
            .take_within_exclusive("|", "|")?
            .trim()
            .parse()
            .map_err(|e| Error::Parse("Physical Writes".to_owned(), ColumnDataError::Integer(e)))?;

        let elapsed: f32 = tok
            .take_within_exclusive("|", "|")?
            .trim()
            .parse()
            .map_err(|e| Error::Parse("Elapsed Time".to_owned(), ColumnDataError::Float(e)))?;
        let elapsed = (elapsed * 1000.0f32).trunc() as u64;

        let statement = tok.take_within_exclusive("|", "|")?.trim().into();
        let command_text = tok.take_within_exclusive("|", "|")?.trim().into();
        let command = tok.take_within_exclusive("|", "|")?.trim().into();
        let login = tok.take_within_exclusive("|", "|")?.trim().into();
        let host = tok.take_within_exclusive("|", "|")?.trim().into();
        let db = tok.take_within_exclusive("|", "|")?.trim().into();
        let program = tok.take_within_exclusive("|", "|")?.trim().into();
        let host_process = tok
            .take_within_exclusive("|", "|")?
            .trim()
            .parse()
            .map_err(|e| Error::Parse("Host Process ID".to_owned(), ColumnDataError::Integer(e)))?;
        let last_request_end = tok.take_within_exclusive("|", "|")?.trim();
        let last_request_end =
            util::utc_unix_timestamp_millis(last_request_end, STUCKQUERY_LOGIN_TIME_FORMAT)?;
        let login_time = tok.take_within_exclusive("|", "|")?.trim();
        let login_time = util::utc_unix_timestamp_millis(login_time, STUCKQUERY_LOGIN_TIME_FORMAT)?;
        let open_txn = tok
            .take_within_exclusive("|", "|")?
            .trim()
            .parse()
            .map_err(|e| Error::Parse("Open Txn".to_owned(), ColumnDataError::Integer(e)))?;

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

#[derive(Debug)]
pub struct BlockingQuery<'a> {
    pub head_blocker: u64,
    pub session_id: u64,
    pub txn_id: u64,
    pub blocking_session_id: u64,
    pub wait_type: Option<WaitType>,
    pub wait_duration: u64,
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
        let head_blocker = tok
            .take_within_exclusive("|", "|")?
            .trim()
            .parse()
            .map_err(|e| {
                Error::Parse(
                    "Head Blocker Session ID".to_owned(),
                    ColumnDataError::Integer(e),
                )
            })?;

        let session_id = tok
            .take_within_exclusive("|", "|")?
            .trim()
            .parse()
            .map_err(|e| Error::Parse("Session ID".to_owned(), ColumnDataError::Integer(e)))?;

        let txn_id = tok
            .take_within_exclusive("|", "|")?
            .trim()
            .parse()
            .map_err(|e| Error::Parse("Txn ID".to_owned(), ColumnDataError::Integer(e)))?;

        let blocking_session_id = tok
            .take_within_exclusive("|", "|")?
            .trim()
            .parse()
            .map_err(|e| {
                Error::Parse(
                    "Blocking Session ID".to_owned(),
                    ColumnDataError::Integer(e),
                )
            })?;

        let wait_type = tok.take_within_exclusive("|", "|")?.trim();
        let wait_type = if wait_type.is_empty() {
            None
        } else {
            Some(WaitType::parse(wait_type))
        };

        let wait_duration = tok
            .take_within_exclusive("|", "|")?
            .trim()
            .parse()
            .map_err(|e| Error::Parse("Wait Duration".to_owned(), ColumnDataError::Integer(e)))?;

        let wait_resource = tok.take_within_exclusive("|", "|")?.trim();
        let wait_resource = if wait_resource.is_empty() {
            None
        } else {
            Some(wait_resource.into())
        };

        let statement_start_offset = tok
            .take_within_exclusive("|", "|")?
            .trim()
            .parse()
            .map_err(|e| {
                Error::Parse(
                    "Statement Start Offset".to_owned(),
                    ColumnDataError::Integer(e),
                )
            })?;

        let statement_end_offset = tok
            .take_within_exclusive("|", "|")?
            .trim()
            .parse()
            .map_err(|e| {
                Error::Parse(
                    "Statement End Offset".to_owned(),
                    ColumnDataError::Integer(e),
                )
            })?;

        let plan_handle = tok.take_within_exclusive("|", "|")?.trim().into();
        let sql_handle = tok.take_within_exclusive("|", "|")?.trim().into();
        let most_recent_sql_handle = tok.take_within_exclusive("|", "|")?.trim().into();
        let level = tok
            .take_within_exclusive("|", "|")?
            .trim()
            .parse()
            .map_err(|e| Error::Parse("Level".to_owned(), ColumnDataError::Integer(e)))?;
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

#[cfg(test)]
pub mod test {
    use std::{assert_matches, ops::Deref};

    use crate::{
        parser::{
            WaitType,
            stuckquery::{
                BlockingQuery, MSSQLStatus, PGSQLQuery, PGSQLState, RunningQuery, StuckqueryParser,
            },
            tokenizer::{Parser, Tokenizer},
        },
        util,
    };

    #[test]
    fn stuckquery_pgsql_single_line() {
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
    fn stuckquery_pgsql_single_table() {
        let map = util::map_file("test/stuckqueries/stuckquery_pgsql_single_table.txt").unwrap();
        let mut parser = StuckqueryParser::try_from(map.deref()).unwrap();
        let result = parser.next();
        assert!(result.is_some());
        let result = result.unwrap();
        assert!(
            result.is_ok(),
            "Error during parsing: {}",
            result.unwrap_err()
        );
        let result = result.unwrap();
        assert_ne!(result.timestamp, 0);
        assert_eq!(result.queries.len(), 35);
    }

    #[test]
    fn stuckquery_pgsql_full_file() {
        let map = util::map_file("test/stuckqueries/stuckquery_pgsql.txt").unwrap();
        let parser = StuckqueryParser::try_from(map.deref()).unwrap();
        let mut count = 0;
        for result in parser {
            assert!(
                result.is_ok(),
                "Error during parsing: {}",
                result.unwrap_err()
            );
            count += 1;
        }
        assert_eq!(count, 41);
    }

    #[test]
    fn stuckquery_mssql_single_running_query() {
        let query = r"|  114         |  suspended  |  35839033162  |  0           |  PAGEIOLATCH_EX  |  5:1:134959573  |  0.000000       |  402.755000    |  144979671      |  5529498   |  2510814  |  1319.037000       |  DBCC SHRINKDATABASE(N'sdpload15140' )                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |                |  DbccFilesCompact  |  SDP-DB-W221\Administrator  |  SDP-DB-W221  |  sdpload15140  |  SQL Server Management Studio          |  24968            |  2026-09-07 23:15:39.013  |  2026-09-07 23:15:25.7    |  1                       |";
        let result = RunningQuery::parse(query);
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
    fn stuckquery_mssql_single_running_query_table() {
        let map = util::map_file("test/stuckqueries/stuckquery_mssql_single_table.txt").unwrap();
        let parser = StuckqueryParser::try_from(map.deref()).unwrap();
        for result in parser {
            assert!(
                result.is_ok(),
                "Error during parsing: {}",
                result.unwrap_err()
            );
            let result = result.unwrap();
            assert_eq!(result.queries.len(), 18);
            assert_ne!(result.timestamp, 0);
        }
    }

    #[test]
    fn stuckquery_mssql_single_blocking_query() {
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
        assert_eq!(result.wait_duration, 0);
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
        let queries = StuckqueryParser::extract_mssql_blocking_queries(&mut tok);
        assert!(
            queries.is_ok(),
            "Error during parsing: {}",
            queries.unwrap_err()
        );
        let queries = queries.unwrap();
        assert_eq!(queries.len(), 35);
    }

    #[test]
    fn stuckquery_mssql_full_file() {
        let map =
            util::map_file("test/stuckqueries/stuckquery_mssql.txt")
                .unwrap();
        let parser = StuckqueryParser::try_from(map.deref()).unwrap();
        let result = parser.flatten().collect::<Vec<_>>();
        assert_eq!(result.len(), 61);
    }
}

pub mod error {
    use std::{
        net::AddrParseError,
        num::{ParseFloatError, ParseIntError},
    };

    use time::error::Parse;

    use crate::{parser::tokenizer, types::TimestampError};

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Invalid Format: {0}")]
        InvalidFormat(#[from] tokenizer::error::Error),
        #[error("Timestamp Parse Error: {0}")]
        TimestampParse(#[from] TimestampError),
        #[error("Unable to find stuckquery table header")]
        UnableToFindTableHeader,
        #[error("Expected Stuckquery Table header to be of length 5 lines")]
        InvalidTableHeader,
        #[error("Unable to detect Stuckquery Table Kind")]
        UnableToDetectTableKind,
        #[error("parse {0} column due to {1}")]
        Parse(String, ColumnDataError),
    }

    #[derive(Debug, thiserror::Error)]
    pub enum ColumnDataError {
        #[error("parse integer due to {0}")]
        Integer(#[from] ParseIntError),
        #[error("parse floating point number due to {0}")]
        Float(#[from] ParseFloatError),
        #[error("parse PGSQL State due to {0}")]
        PGSQLState(#[from] PGSQLStateParse),
        #[error("parse MSSQL Status due to {0}")]
        MSSQLStatus(#[from] MSSQLStatusParse),
        #[error("parse UTC timestamp due to {0}")]
        Time(#[from] Parse),
        #[error("parse IP Address due to {0}")]
        IpV4Addr(#[from] AddrParseError),
    }

    #[derive(Debug, thiserror::Error)]
    pub enum PGSQLStateParse {
        #[error("Invalid PGSQL state: {0}")]
        UnknownState(String),
    }

    #[derive(Debug, thiserror::Error)]
    pub enum MSSQLStatusParse {
        #[error("Invalid MSSQL status: {0}")]
        UnknownStatus(String),
    }
}
