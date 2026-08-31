use std::net::Ipv4Addr;

use time::{Date, PrimitiveDateTime, Time, UtcDateTime, macros::format_description};
use tracing::warn;

use crate::{
    error::stuckquery::{self, MSSQLParse, PGParse},
    ingest::kind::DBKind,
    parser::scanner::Scanner,
    util::ToUnixMillis,
};

#[derive(Debug)]
pub enum StuckQueryTable<'a> {
    PGSQL(PGSQLTable<'a>),
    MSSQL(MSSQLTable<'a>),
}

#[derive(Debug)]
pub struct PGSQLTable<'a> {
    pub queries: Vec<PGSQLQuery<'a>>,
    pub timestamp: u64,
}

#[derive(Debug)]
pub struct PGSQLQuery<'a> {
    pub db_name: &'a str,
    pub query: &'a str,
    pub application_name: &'a str,
    pub client_hostname: Option<&'a str>,
    pub pid: u64,
    pub query_time_ms: u64,
    pub txn_time_ms: u64,
    pub last_state_change: u64,
    pub client_address: Option<Ipv4Addr>,
    pub client_port: Option<u16>,
    pub state: State<'a>,
    pub waiting: bool,
}

#[derive(Debug)]
pub enum State<'a> {
    Active,
    Idle,
    Unknown(&'a str),
}

static PGSQL_STATE_CHANGE_FORMAT: &[time::format_description::FormatItem<'static>] = format_description!(
    "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond][offset_hour sign:mandatory]:[offset_minute]"
);

static PGSQL_TIMESTAMP_TIME_FORMAT: &[time::format_description::FormatItem<'static>] =
    format_description!("[hour]:[minute]:[second].[subsecond]");

static PGSQL_TIMESTAMP_DATE_FORMAT: &[time::format_description::FormatItem<'static>] =
    format_description!("[day]-[month]-[year]");

impl<'a> StuckQueryTable<'a> {
    pub fn parse(kind: DBKind, value: &'a str) -> Result<Self, stuckquery::Error> {
        match kind {
            DBKind::PGSQL => Ok(StuckQueryTable::PGSQL(PGSQLTable::try_from(value)?)),
            DBKind::MSSQL => Ok(StuckQueryTable::MSSQL(MSSQLTable::try_from(value)?)),
        }
    }
}

impl<'a> TryFrom<&'a str> for StuckQueryTable<'a> {
    type Error = stuckquery::Error;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        DBKind::detect(value.as_bytes())
            .map_err(stuckquery::Error::UnableToDetectKind)
            .and_then(|inner| match inner {
                DBKind::PGSQL => Ok(StuckQueryTable::PGSQL(PGSQLTable::try_from(value)?)),
                DBKind::MSSQL => Ok(StuckQueryTable::MSSQL(MSSQLTable::try_from(value)?)),
            })
    }
}

impl<'a> PGSQLTable<'a> {
    pub fn extract_timestamp(header: &'a str) -> Result<u64, PGParse> {
        let mut scanner = Scanner::new(header);
        let time = scanner
            .take_within("[", "]")
            .map_err(|_| PGParse::TableHeaderMetaExtraction)?;
        let date = scanner
            .take_within("[", "]")
            .map_err(|_| PGParse::TableHeaderMetaExtraction)?;
        let parsed_time =
            Time::parse(time, PGSQL_TIMESTAMP_TIME_FORMAT).map_err(PGParse::TimestampParse)?;
        let parsed_date =
            Date::parse(date, PGSQL_TIMESTAMP_DATE_FORMAT).map_err(PGParse::TimestampParse)?;
        let datetime = PrimitiveDateTime::new(parsed_date, parsed_time);
        Ok(datetime.to_unix_millis().unwrap_or(0))
    }
}

impl<'a> TryFrom<&'a str> for PGSQLTable<'a> {
    type Error = PGParse;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        let mut queries = Vec::new();
        let header = scanner
            .take_until("\n")
            .ok_or_else(|| PGParse::TableHeaderExtraction)?;

        let timestamp = Self::extract_timestamp(header)?;

        for _ in 0..7 {
            let line = scanner.take_until("\n");
            if line.is_none() {
                warn!("Table is empty, skipping parsing");
                return Err(PGParse::TableExtraction);
            }
        }

        while let Some(line) = scanner.take_until("\n") {
            if !line.trim().starts_with("|") {
                break;
            }
            match PGSQLQuery::try_from(line) {
                Ok(x) => queries.push(x),
                Err(e) => warn!("Cannot parse {line} due to {e:?}"),
            }
        }
        Ok(Self { queries, timestamp })
    }
}

impl<'a> TryFrom<&'a str> for PGSQLQuery<'a> {
    type Error = PGParse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        let pid = scanner
            .take_within_exclusive("|", "|")
            .map_err(PGParse::PidExtraction)?
            .trim()
            .parse()
            .map_err(PGParse::InvalidPID)?;
        let query_time = scanner
            .take_within_exclusive("|", "|")
            .map_err(PGParse::QueryTimeExtraction)?
            .trim()
            .parse::<f32>()
            .map_err(PGParse::InvalidQueryTime)?;
        let query_time_ms = (query_time * 1000.0f32).trunc() as u64;

        let txn_time = scanner
            .take_within_exclusive("|", "|")
            .map_err(PGParse::TransactionTimeExtraction)?
            .trim();
        let txn_time = if txn_time.is_empty() {
            Ok(0.0)
        } else {
            txn_time.parse().map_err(PGParse::InvalidTransactionTime)
        }?;
        let txn_time_ms = (txn_time * 1000.0f32).trunc() as u64;

        let db_name = scanner
            .take_within_exclusive("|", "|")
            .map_err(PGParse::DatabaseNameExtraction)?
            .trim();

        let state = scanner
            .take_within_exclusive("|", "|")
            .map_err(PGParse::StateExtraction)?
            .trim();
        let state = State::from(state);

        let waiting = scanner
            .take_within_exclusive("|", "|")
            .map_err(PGParse::WaitingExtraction)?
            .trim();
        let waiting = match waiting {
            "f" => false,
            "t" => true,
            _ => {
                return Err(PGParse::InvalidWaitingState {
                    got: String::from(waiting),
                });
            }
        };

        let query = scanner
            .take_within_exclusive("|", "|")
            .map_err(PGParse::QueryExtraction)?
            .trim();

        let state_change = scanner
            .take_within_exclusive("|", "|")
            .map_err(PGParse::StateChangeExtraction)?
            .trim();

        let state_change = UtcDateTime::parse(state_change, PGSQL_STATE_CHANGE_FORMAT)
            .map_err(PGParse::InvalidStateChange)?
            .unix_timestamp_nanos()
            / 1_000_000;
        let last_state_change = state_change as u64;

        let application_name = scanner
            .take_within_exclusive("|", "|")
            .map_err(PGParse::ApplicationNameExtraction)?
            .trim();

        let client_address = scanner
            .take_within_exclusive("|", "|")
            .map_err(PGParse::ClientAddressExtraction)?
            .trim();

        let client_address = if client_address.is_empty() {
            None
        } else {
            Some(client_address.parse()?)
        };

        let client_hostname = scanner
            .take_within_exclusive("|", "|")
            .map_err(PGParse::ClientHostnameExtraction)?;
        let client_hostname = if client_hostname.trim().is_empty() {
            None
        } else {
            Some(client_hostname)
        };

        let client_port = scanner
            .take_within_exclusive("|", "|")
            .map_err(PGParse::ClientPortExtraction)?
            .trim();
        let client_port = if !client_port.is_empty() {
            Some(client_port.parse().map_err(PGParse::InvalidClientPort)?)
        } else {
            None
        };

        Ok(Self {
            pid,
            client_port,
            client_hostname,
            db_name,
            application_name,
            state,
            waiting,
            client_address,
            query_time_ms,
            query,
            txn_time_ms,
            last_state_change,
        })
    }
}

impl<'a> State<'a> {
    pub fn is_active(&self) -> bool {
        if let Self::Active = self {
            return true;
        } else {
            false
        }
    }
}

impl<'a> From<&'a str> for State<'a> {
    fn from(value: &'a str) -> Self {
        match value {
            "active" => State::Active,
            "idle in transaction" => State::Idle,
            unknown => State::Unknown(unknown),
        }
    }
}

static LAST_REQUEST_FORMAT: &[time::format_description::FormatItem<'static>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]");

pub static MSSQL_TIMESTAMP_TIME_FORMAT: &[time::format_description::FormatItem<'static>] =
    format_description!("[hour]:[minute]:[second].[subsecond]");

pub static MSSQL_TIMESTAMP_DATE_FORMAT: &[time::format_description::FormatItem<'static>] =
    format_description!("[day]-[month]-[year]");

#[derive(Debug)]
pub struct MSSQLTable<'a> {
    pub queries: Vec<MSSQLQuery<'a>>,
    pub timestamp: u64,
}

#[derive(Debug)]
pub struct MSSQLQuery<'a> {
    pub session_id: u64,
    pub status: Status<'a>,
    pub txn_id: u64,
    pub blocked_by: u64,
    pub wait_type: Option<&'a str>,
    pub wait_resource: Option<&'a str>,
    pub wait_time_ms: u64,
    pub cpu_time_ms: u64,
    pub logical_reads: u64,
    pub physical_reads: u64,
    pub physical_writes: u64,
    pub elapsed_time_ms: u64,
    pub statement: &'a str,
    pub command_text: Option<&'a str>,
    pub command: Option<&'a str>,
    pub login_name: &'a str,
    pub host_name: &'a str,
    pub db_name: &'a str,
    pub program_name: &'a str,
    pub host_process_id: u64,
    pub last_request_end_ms: u64,
    pub login_time_ms: u64,
    pub open_transaction_count: u64,
}

#[derive(Debug)]
pub enum Status<'a> {
    Running,
    Runnable,
    Suspended,
    Unknown(&'a str),
}

impl<'a> From<&'a str> for Status<'a> {
    fn from(value: &'a str) -> Self {
        match value {
            "running" => Self::Running,
            "runnable" => Self::Runnable,
            "suspended" => Self::Suspended,
            s => Self::Unknown(s),
        }
    }
}

impl<'a> MSSQLTable<'a> {
    fn extract_timestamp(header: &str) -> Result<u64, MSSQLParse> {
        let mut scanner = Scanner::new(header);
        let time = scanner
            .take_within("[", "]")
            .map_err(|_| MSSQLParse::TableHeaderMetaExtraction)?;
        let date = scanner
            .take_within("[", "]")
            .map_err(|_| MSSQLParse::TableHeaderMetaExtraction)?;
        let parsed_time =
            Time::parse(time, MSSQL_TIMESTAMP_TIME_FORMAT).map_err(MSSQLParse::TimestampParse)?;
        let parsed_date =
            Date::parse(date, MSSQL_TIMESTAMP_DATE_FORMAT).map_err(MSSQLParse::TimestampParse)?;
        let datetime = PrimitiveDateTime::new(parsed_date, parsed_time);
        Ok(datetime.to_unix_millis().unwrap_or(0))
    }
}

impl<'a> TryFrom<&'a str> for MSSQLTable<'a> {
    type Error = MSSQLParse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut queries = Vec::new();
        let mut scanner = Scanner::new(value);
        let header = scanner.take_until("\n");
        let timestamp = header
            .ok_or(MSSQLParse::TableHeaderExtraction)
            .map(Self::extract_timestamp)??;

        for _ in 0..7 {
            if scanner.take_until("\n").is_none() {
                return Err(MSSQLParse::TableExtraction);
            }
        }

        while let Some(line) = scanner.take_until("\n") {
            if !line.trim().starts_with("|") {
                break;
            }

            match MSSQLQuery::try_from(line) {
                Ok(x) => queries.push(x),
                Err(e) => warn!("Cannot parse MSSQL query {line} due to {e:?}"),
            }
        }
        Ok(Self { timestamp, queries })
    }
}

impl<'a> TryFrom<&'a str> for MSSQLQuery<'a> {
    type Error = MSSQLParse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        let session_id = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSSQLParse::SessionIDExtraction)?
            .trim()
            .parse::<u64>()
            .map_err(MSSQLParse::SessionIDParse)?;

        let status = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSSQLParse::StatusExtraction)?
            .trim();
        let status = Status::from(status);

        let txn_id = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSSQLParse::TransactionIDExtraction)?
            .trim()
            .parse::<u64>()
            .map_err(MSSQLParse::TransactionIDParse)?;

        let blocked_by = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSSQLParse::BlockedByExtraction)?
            .trim()
            .parse::<u64>()
            .map_err(MSSQLParse::BlockedByParse)?;

        let wait_type = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSSQLParse::WaitTypeExtraction)?
            .trim();
        let wait_type = if wait_type.is_empty() {
            None
        } else {
            Some(wait_type)
        };

        let wait_resource = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSSQLParse::WaitResourceExtraction)?
            .trim();
        let wait_resource = if wait_resource.is_empty() {
            None
        } else {
            Some(wait_resource)
        };

        let wait_time = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSSQLParse::WaitTimeExtraction)?
            .trim()
            .parse::<f32>()
            .map_err(MSSQLParse::WaitTimeParse)?;

        let wait_time_ms = (wait_time * 1000.0f32).trunc() as u64;

        let cpu_time = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSSQLParse::CPUTimeExtraction)?
            .trim()
            .parse::<f32>()
            .map_err(MSSQLParse::CPUTimeParse)?;

        let cpu_time_ms = (cpu_time * 1000.0f32).trunc() as u64;

        let logical_reads = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSSQLParse::LogicalReadsExtraction)?
            .trim()
            .parse::<u64>()
            .map_err(MSSQLParse::LogicalReadsParse)?;

        let physical_reads = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSSQLParse::PhysicalReadsExtraction)?
            .trim()
            .parse::<u64>()
            .map_err(MSSQLParse::PhysicalReadsParse)?;

        let physical_writes = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSSQLParse::PhysicalWritesExtraction)?
            .trim()
            .parse::<u64>()
            .map_err(MSSQLParse::PhysicalWritesParse)?;

        let elapsed_time = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSSQLParse::ElapsedTimeExtraction)?
            .trim()
            .parse::<f32>()
            .map_err(MSSQLParse::ElapsedTimeParse)?;

        let elapsed_time_ms = (elapsed_time * 1000.0f32).trunc() as u64;

        let statement = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSSQLParse::StatementExtraction)?
            .trim();

        let command_text = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSSQLParse::CommandTextExtraction)?
            .trim();

        let command_text = if command_text.is_empty() {
            None
        } else {
            Some(command_text)
        };

        let command = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSSQLParse::CommandExtraction)?
            .trim();

        let command = if command.is_empty() {
            None
        } else {
            Some(command)
        };

        let login_name = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSSQLParse::LoginNameExtraction)?
            .trim();

        let host_name = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSSQLParse::HostNameExtraction)?
            .trim();

        let db_name = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSSQLParse::DatabaseNameExtraction)?
            .trim();

        let program_name = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSSQLParse::ProgramNameExtraction)?
            .trim();

        let host_process_id = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSSQLParse::HostProcessIDExtraction)?
            .trim()
            .parse::<u64>()
            .map_err(MSSQLParse::HostProcessIDParse)?;

        let last_request_end_ms = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSSQLParse::LastRequestEndExtraction)?
            .trim();

        let last_request_end_ms =
            PrimitiveDateTime::parse(last_request_end_ms, LAST_REQUEST_FORMAT)
                .map_err(MSSQLParse::LastRequestEndParse)?
                .to_unix_millis()
                .unwrap_or(0);

        let login_time_ms = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSSQLParse::LoginTimeExtraction)?
            .trim();

        let login_time_ms = PrimitiveDateTime::parse(login_time_ms, LAST_REQUEST_FORMAT)
            .map_err(MSSQLParse::LoginTimeParse)?
            .to_unix_millis()
            .unwrap_or(0);

        let open_transaction_count = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSSQLParse::OpenTransactionCountExtraction)?
            .trim()
            .parse::<u64>()
            .map_err(MSSQLParse::OpenTransactionCountParse)?;

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
            physical_reads,
            physical_writes,
            elapsed_time_ms,
            statement,
            command_text,
            command,
            login_name,
            host_name,
            db_name,
            program_name,
            host_process_id,
            last_request_end_ms,
            login_time_ms,
            open_transaction_count,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ingest::stuckquery::StuckQueryIterator,
        parser::stuckquery::{DBKind, MSSQLTable, PGSQLQuery, PGSQLTable},
        util,
    };

    #[test]
    fn stuckquery_mssql_test_empty_table() {
        let map = util::map_file("test/stuckquery_mssql_empty_table.txt").unwrap();
        let file = str::from_utf8(&map).unwrap_or("");
        let table = MSSQLTable::try_from(file);
        assert!(
            table.is_ok(),
            "Error during parsing table: {:?}",
            table.unwrap_err()
        );

        let table = table.unwrap();
        assert_eq!(table.queries.len(), 0);
    }

    #[test]
    fn stuckquery_mssql_test_full_table() {
        let map = util::map_file("test/stuckquery_mssql_full_table.txt").unwrap();
        let file = str::from_utf8(&map).unwrap_or("");
        let table = MSSQLTable::try_from(file);
        assert!(
            table.is_ok(),
            "Error during parsing table: {:?}",
            table.unwrap_err()
        );

        let table = table.unwrap();
        assert_eq!(table.queries.len(), 7);
    }

    #[test]
    fn stuckquery_pgsql_parse_single_row_no_client_info() {
        let input = "|  11532  |  4542.352065     |  4542.359728   |  servicedesk  |  active               |  f        |  SELECT count(*) as orphanentries FROM  notes  WHERE	 (notesid  not in 	(  	SELECT notesid 	FROM  workordernotes  	))  limit 50000                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |  2026-02-18 15:02:14.229876+05:30  |  PostgreSQL JDBC Driver  |                  |                   |               |";
        let output = PGSQLQuery::try_from(input);
        assert!(output.is_ok(), "{}", output.unwrap_err());
        let output = output.unwrap();
        println!("output: {output:#?}");
    }

    #[test]
    fn stuckquery_pgsql_parse_single_row_with_client_info() {
        let input = "|  16860  |  1931.755461     |  1931.766792   |  servicedesk  |  active               |  f        |  SELECT count(*) as orphanentries FROM  notes  WHERE	 (notesid  not in 	(  	SELECT notesid 	FROM  workordernotes  	))  limit 50000                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |  2026-02-18 15:45:44.739882+05:30  |  PostgreSQL JDBC Driver  |  127.0.0.1       |                   |  60320        |";
        let output = PGSQLQuery::try_from(input);
        assert!(output.is_ok(), "{}", output.unwrap_err());
        let output = output.unwrap();
        println!("output: {output:#?}");
    }

    #[test]
    fn stuckquery_pgsql_parse_single_row_negative_timestamp() {
        let input = "|  13280  |  -0.001069       |  -0.001069     |  servicedesk  |  active  |  f        |  SELECT PurchaseOrder.PURCHASEORDERID AS \"PurchaseOrderID\", PurchaseOrder.POCUSTOMID AS \"POCustomID\", PurchaseOrder.PONAME AS \"POName\", PurchaseOrder.OWNERID, PurchaseOrder.DATEORDERED, AaaUser.USER_ID, AaaUser.FIRST_NAME AS \"Owner\", PurchaseOrder.DATEREQUIRED AS \"Required By\", PurchaseOrder.STATUSID, POStatus.STATUSID, POStatus.STATUSNAME AS \"Status\" FROM PurchaseOrder INNER JOIN POStatus ON PurchaseOrder.STATUSID=POStatus.STATUSID LEFT JOIN AaaUser ON PurchaseOrder.OWNERID=AaaUser.USER_ID LEFT JOIN WorkOrderToPurchaseOrder ON PurchaseOrder.PURCHASEORDERID=WorkOrderToPurchaseOrder.PURCHASEORDERID WHERE  (( PurchaseOrder.HELPDESKID = 1 ) AND ( WorkOrderToPurchaseOrder.WORKORDERID = 3485639 ))  ORDER BY 5 DESC LIMIT 25  |  2026-02-18 16:22:19.2058+05:30    |  PostgreSQL JDBC Driver  |  127.0.0.1       |                   |  58517        |";
        let output = PGSQLQuery::try_from(input);
        assert!(output.is_ok(), "{}", output.unwrap_err());
        let output = output.unwrap();
        println!("output: {output:#?}");
    }

    #[test]
    fn stuckquery_pgsql_parse_table() {
        let map = util::map_file("test/stuckquery_pgsql_single_table.txt").unwrap();
        let kind = DBKind::detect(&map).unwrap();
        assert_eq!(kind, DBKind::PGSQL);
        let table = StuckQueryIterator::new(kind, &map).next();
        assert!(table.is_some(), "Failed to stream tables");
        let table = table.unwrap();
        let table = PGSQLTable::try_from(table);
        assert!(table.is_ok(), "Error: {:?}", table.unwrap_err());
        let table = table.unwrap();
        assert_eq!(table.queries.len(), 24);
    }

    #[test]
    fn stuckquery_pgsql_parse_full_file() {
        let map = util::map_file("test/stuckqueries_pgsql_full.txt").unwrap();
        let kind = DBKind::detect(&map).unwrap();
        assert_eq!(kind, DBKind::PGSQL);
        for table in StuckQueryIterator::new(kind, &map) {
            let table = PGSQLTable::try_from(table);
            assert!(table.is_ok(), "Error: {:?}", table.unwrap_err());
            let table = table.unwrap();
            assert_ne!(table.queries.len(), 0);
        }
    }

    #[test]
    fn stuckquery_pgsql_parse_empty_table() {
        let map = util::map_file("test/stuckquery_pgsql_empty_table.txt").unwrap();
        let kind = DBKind::detect(&map).unwrap();
        assert_eq!(kind, DBKind::PGSQL);
        for table in StuckQueryIterator::new(kind, &map) {
            let table = PGSQLTable::try_from(table);
            assert!(table.is_ok(), "Error: {:?}", table.unwrap_err());
            let table = table.unwrap();
            assert_eq!(table.queries.len(), 0);
        }
    }
}
