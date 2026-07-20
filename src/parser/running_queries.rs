use std::{net::Ipv4Addr, num::ParseIntError};

use time::{Date, PrimitiveDateTime, Time, UtcDateTime, macros::format_description};
use tracing::warn;

use crate::{
    error::running_query::{Error, MSParse, PGParse},
    ingest::running_queries::Entry,
    parser::scanner::Scanner,
    util::ToUnixMillis,
};

static PGSQL_STATE_CHANGE_FORMAT: &[time::format_description::FormatItem<'static>] = format_description!(
    "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond][offset_hour sign:mandatory]:[offset_minute]"
);

static TIMESTAMP_TIME_FORMAT: &[time::format_description::FormatItem<'static>] =
    format_description!("[hour]:[minute]:[second].[subsecond]");

static TIMESTAMP_DATE_FORMAT: &[time::format_description::FormatItem<'static>] =
    format_description!("[day]-[month]-[year]");

static LAST_REQUEST_FORMAT: &[time::format_description::FormatItem<'static>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]");

pub enum RunningQueryTable<'a> {
    MSSQL(MSSQLRunningQueryTable<'a>),
    PGSQL(PGSQLRunningQueryTable<'a>),
}

#[derive(Debug)]
pub struct PGSQLRunningQuery<'a> {
    pub pid: u64,
    pub query_time_ms: u64,
    pub txn_time_ms: u64,
    pub db_name: &'a str,
    pub state: State<'a>,
    pub waiting: bool,
    pub query: &'a str,
    pub last_state_change: u64,
    pub application_name: &'a str,
    pub client_address: Option<Ipv4Addr>,
    pub client_port: Option<u16>,
    pub client_hostname: Option<&'a str>,
}

#[derive(Debug)]
pub struct PGSQLRunningQueryTable<'a> {
    pub queries: Vec<PGSQLRunningQuery<'a>>,
    pub timestamp: u64,
}

#[derive(Debug)]
pub enum State<'a> {
    Active,
    Idle,
    Unknown(&'a str),
}

#[derive(Debug)]
pub enum MSSQLRunningQueryTable<'a> {
    SPWho2(SPWho2Table<'a>),
    RunningQuery(MSSQLCurrentRunningQueryTable<'a>),
}

#[derive(Debug)]
pub struct MSSQLCurrentRunningQueryTable<'a> {
    pub timestamp: u64,
    pub queries: Vec<MSSQLQuery<'a>>,
}

#[derive(Debug)]
pub struct SPWho2Table<'a> {
    pub timestamp: u64,
    pub entries: Vec<SPWho2Entry>,
}

#[derive(Debug)]
pub struct SPWho2Entry<'a> {
    pub spid: u64,
    pub status: SPWho2Status<'a>,
    pub login: &'a str,
    pub hostname: &'a str,
    pub blocked_by: Option<u64>,
    pub db_name: &'a str,
    pub command: &'a str,
    pub cpu_time: u64,
    pub disk_io: u64,
    pub last_batch: &'a str,
    pub program_name: &'a str,
    pub request_id: &'a str,
}

#[derive(Debug)]
pub enum SPWho2Status<'a> {
    Sleeping,
    Background,
    Runnable,
    Unknown(&'a str),
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
    Sleeping,
    Background,
    Unknown(&'a str),
}

impl<'a> From<&'a str> for Status<'a> {
    fn from(value: &'a str) -> Self {
        match value {
            "sleeping" => Status::Sleeping,
            "runnable" => Status::Runnable,
            "running" => Status::Running,
            "background" => Status::Background,
            "suspended" => Status::Suspended,
            unknown => Status::Unknown(unknown),
        }
    }
}

impl<'a> PGSQLRunningQueryTable<'a> {
    fn extract_timestamp(meta: &str) -> Result<u64, PGParse> {
        let mut scanner = Scanner::new(meta);
        let time = scanner
            .take_within("[", "]")
            .map_err(PGParse::TableHeaderMetaExtraction)?;
        let date = scanner
            .take_within("[", "]")
            .map_err(PGParse::TableHeaderMetaExtraction)?;
        let parsed_time =
            Time::parse(time, TIMESTAMP_TIME_FORMAT).map_err(PGParse::TableHeaderMetaTimeParse)?;
        let parsed_date =
            Date::parse(date, TIMESTAMP_DATE_FORMAT).map_err(PGParse::TableHeaderMetaTimeParse)?;
        let timestamp = PrimitiveDateTime::new(parsed_date, parsed_time).to_unix_millis();
        Ok(timestamp.unwrap_or(0))
    }
}

impl<'a> MSSQLRunningQueryTable<'a> {
    fn extract_timestamp(meta: &str) -> Result<u64, MSParse> {
        let mut scanner = Scanner::new(meta);
        let time = scanner
            .take_within("[", "]")
            .map_err(MSParse::TableHeaderMetaExtraction)?;
        let date = scanner
            .take_within("[", "]")
            .map_err(MSParse::TableHeaderMetaExtraction)?;
        let parsed_time =
            Time::parse(time, TIMESTAMP_TIME_FORMAT).map_err(MSParse::TableHeaderMetaParse)?;
        let parsed_date =
            Date::parse(date, TIMESTAMP_DATE_FORMAT).map_err(MSParse::TableHeaderMetaParse)?;
        let timestamp = PrimitiveDateTime::new(parsed_date, parsed_time).to_unix_millis();
        Ok(timestamp.unwrap_or(0))
    }
}

impl<'a> TryFrom<(Entry<'a>, Entry<'a>)> for PGSQLRunningQueryTable<'a> {
    type Error = PGParse;

    fn try_from(value: (Entry<'a>, Entry<'a>)) -> Result<Self, Self::Error> {
        let (meta, table) = match value {
            (Entry::Meta(meta), Entry::Table(table)) => (meta, table),
            _ => {
                return Err(PGParse::InvalidEntryType);
            }
        };

        let timestamp = Self::extract_timestamp(meta)?;
        let mut scanner = Scanner::new(table);
        for _ in 0..5 {
            let line = scanner.take_until("\n");
            if line.is_none() {
                warn!("Table is empty, skipping parsing");
                return Ok(Self {
                    timestamp,
                    queries: Vec::new(),
                });
            }
        }

        let mut queries = Vec::with_capacity(40);
        while let Some(line) = scanner.take_until("\n") {
            if !line.trim().starts_with("|") {
                break;
            }

            match PGSQLRunningQuery::try_from(line) {
                Ok(x) => queries.push(x),
                Err(e) => warn!("Cannot parse {line} due to {e:?}"),
            }
        }
        Ok(Self { timestamp, queries })
    }
}

impl<'a> TryFrom<&'a str> for PGSQLRunningQuery<'a> {
    type Error = PGParse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        let pid = scanner
            .take_within_exclusive("|", "|")
            .map_err(PGParse::PIDExtraction)?
            .trim()
            .parse()
            .map_err(PGParse::PIDParse)?;

        let query_time_ms: f32 = scanner
            .take_within_exclusive("|", "|")
            .map_err(PGParse::QueryTimeExtraction)?
            .trim()
            .parse()
            .map_err(PGParse::QueryTimeParse)?;

        let query_time_ms = (query_time_ms * 1000.0f32).trunc() as u64;
        let txn_time = scanner
            .take_within_exclusive("|", "|")
            .map_err(PGParse::TransactionTimeExtraction)?
            .trim();

        let txn_time_ms = if txn_time.is_empty() {
            Ok(0.0)
        } else {
            txn_time.parse().map_err(PGParse::TransactionTimeParse)
        }?;

        let txn_time_ms = (txn_time_ms * 1000.0f32).trunc() as u64;

        let db_name = scanner
            .take_within_exclusive("|", "|")
            .map_err(PGParse::DatabaseNameExtraction)?
            .trim();

        let state = scanner
            .take_within_exclusive("|", "|")
            .map_err(PGParse::StateExtraction)?
            .trim();

        let state = State::from(state);

        let waiting = match scanner
            .take_within_exclusive("|", "|")
            .map_err(PGParse::WaitingExtraction)?
            .trim()
        {
            "f" => false,
            "t" => true,
            waiting => {
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
            .map_err(PGParse::LastStateChangeParse)?
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
            Some(
                client_address
                    .parse()
                    .map_err(PGParse::ClientAddressParsing)?,
            )
        };

        let client_hostname = scanner
            .take_within_exclusive("|", "|")
            .map_err(PGParse::ClientHostExtraction)?
            .trim();

        let client_hostname = if client_hostname.is_empty() {
            None
        } else {
            Some(client_hostname)
        };

        let client_port = scanner
            .take_within_exclusive("|", "|")
            .map_err(PGParse::ClientPortExtraction)?
            .trim();
        let client_port = if client_port.is_empty() {
            None
        } else {
            Some(client_port.parse().map_err(PGParse::ClientPortParsing)?)
        };

        Ok(Self {
            pid,
            query_time_ms,
            txn_time_ms,
            db_name,
            state,
            waiting,
            query,
            last_state_change,
            application_name,
            client_address,
            client_port,
            client_hostname,
        })
    }
}

impl<'a> From<&'a str> for State<'a> {
    fn from(value: &'a str) -> Self {
        match value {
            "active" => Self::Active,
            "idle in transaction" => State::Idle,
            unknown => State::Unknown(unknown),
        }
    }
}

impl<'a> TryFrom<&'a str> for MSSQLQuery<'a> {
    type Error = MSParse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        let session_id = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::SessionIDExtraction)?
            .trim()
            .parse()
            .map_err(MSParse::SessionID)?;

        let status = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::StatusExtraction)?
            .trim();

        let status = Status::from(status);
        let txn_id = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::TxnIDExtraction)?
            .trim()
            .parse()
            .map_err(MSParse::TxnID)?;

        let blocked_by = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::BlockedByExtraction)?
            .trim()
            .parse()
            .map_err(MSParse::TxnID)?;

        let wait_type = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::WaitTypeExtraction)?
            .trim();

        let wait_type = if wait_type.is_empty() {
            None
        } else {
            Some(wait_type)
        };

        let wait_resource = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::WaitResourceExtraction)?
            .trim();

        let wait_resource = if wait_resource.is_empty() {
            None
        } else {
            Some(wait_resource)
        };

        let wait_time_ms = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::WaitTimeExtraction)?
            .trim()
            .parse::<f32>()
            .map_err(MSParse::WaitTime)?;

        let wait_time_ms = (wait_time_ms * 1000.0f32).trunc() as u64;

        let cpu_time_ms = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::CPUTimeExtraction)?
            .trim()
            .parse::<f32>()
            .map_err(MSParse::CPUTime)?;

        let cpu_time_ms = (cpu_time_ms * 1000.0f32).trunc() as u64;

        let logical_reads = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::LogicalReadsExtraction)?
            .trim()
            .parse()
            .map_err(MSParse::LogicalReads)?;

        let physical_reads = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::PhysicalReadsExtraction)?
            .trim()
            .parse()
            .map_err(MSParse::PhysicalReads)?;

        let physical_writes = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::PhysicalWritesExtraction)?
            .trim()
            .parse()
            .map_err(MSParse::PhysicalWrites)?;

        let elapsed_time_ms = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::ElapsedTimeExtraction)?
            .trim()
            .parse::<f32>()
            .map_err(MSParse::ElapsedTime)?;

        let elapsed_time_ms = (elapsed_time_ms * 1000.0f32).trunc() as u64;

        let statement = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::StatementExtraction)?
            .trim();

        let command_text = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::CommandTextExtraction)?
            .trim();

        let command_text = if command_text.is_empty() {
            None
        } else {
            Some(command_text)
        };

        let command = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::CommandExtraction)?
            .trim();

        let command = if command.is_empty() {
            None
        } else {
            Some(command)
        };

        let login_name = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::LoginNameExtraction)?
            .trim();

        let host_name = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::HostNameExtraction)?
            .trim();

        let db_name = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::DBNameExtraction)?
            .trim();

        let program_name = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::ProgramNameExtraction)?
            .trim();

        let host_process_id = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::HostProcessIDExtraction)?
            .trim()
            .parse()
            .map_err(MSParse::HostProcessID)?;

        let last_request_end_ms = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::LastRequestEndExtraction)?
            .trim();

        let last_request_end_ms =
            PrimitiveDateTime::parse(last_request_end_ms, LAST_REQUEST_FORMAT)
                .map_err(MSParse::LastRequestEnd)?
                .to_unix_millis()
                .unwrap_or(0);

        let login_time_ms = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::LoginTimeMSExtraction)?
            .trim();

        let login_time_ms = PrimitiveDateTime::parse(login_time_ms, LAST_REQUEST_FORMAT)
            .map_err(MSParse::LoginTimeMS)?
            .to_unix_millis()
            .unwrap_or(0);

        let open_transaction_count = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::OpenTransactionCountExtraction)?
            .trim()
            .parse()
            .map_err(MSParse::OpenTransactionCount)?;

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

impl<'a> TryFrom<(Entry<'a>, Entry<'a>)> for MSSQLRunningQueryTable<'a> {
    type Error = MSParse;

    fn try_from(value: (Entry<'a>, Entry<'a>)) -> Result<Self, Self::Error> {
        let (meta, table) = match value {
            (Entry::Meta(meta), Entry::Table(table)) => (meta, table),
            _ => {
                return Err(MSParse::InvalidEntryType);
            }
        };

        let timestamp = MSSQLRunningQueryTable::extract_timestamp(meta)?;
        let mut scanner = Scanner::new(table);
        let _ = scanner.take_until("\n");
        let table_header = scanner
            .take_within("|", "|")
            .map_err(MSParse::TableHeaderExtraction)?
            .trim();
        let _ = scanner.take_until("\n");
        scanner.skip_whitespace();

        match table_header {
            "sp Who2" => todo!(),
            "Currently Running Queries" => {
                for _ in 0..3 {
                    let line = scanner.take_until("\n");
                    if line.is_none() {
                        warn!("Table is empty, skipping parsing");
                        return Ok(Self::RunningQuery(MSSQLCurrentRunningQueryTable {
                            timestamp,
                            queries: Vec::new(),
                        }));
                    }
                }

                let mut queries = Vec::with_capacity(40);
                while let Some(line) = scanner.take_until("\n") {
                    if !line.trim().starts_with("|") {
                        break;
                    }

                    match MSSQLQuery::try_from(line) {
                        Ok(query) => queries.push(query),
                        Err(e) => warn!("Cannot parse MSSQL query due to {e:?}"),
                    }
                }
                return Ok(Self::RunningQuery(MSSQLCurrentRunningQueryTable {
                    timestamp,
                    queries,
                }));
            }
            unknown => {
                warn!("Unrecognized MSSQL Running Query Table Kind: {unknown}");
                return Err(MSParse::InvalidTableHeader(unknown.to_owned()));
            }
        };
    }
}

impl<'a> TryFrom<&'a str> for SPWho2Entry<'a> {
    type Error = MSParse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        let spid = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::SpWho2ExtractionSPID)?
            .trim()
            .parse().map_err(MSParse::SpWho2ParseSPID)?;

        Ok(Self {
            spid,
            status: todo!(),
            login: todo!(),
            hostname: todo!(),
            blocked_by: todo!(),
            db_name: todo!(),
            command: todo!(),
            cpu_time: todo!(),
            disk_io: todo!(),
            last_batch: todo!(),
            program_name: todo!(),
            request_id: todo!(),
        })
        todo!()
    }
}

pub struct RunningQueryParser<T>(pub T);

impl<'a, T> RunningQueryParser<T> {
    pub fn new(iter: T) -> Self
    where
        T: Iterator<Item = Entry<'a>>,
    {
        Self(iter)
    }
}

impl<'a> RunningQueryTable<'a> {
    pub fn extract_timestamp(meta: Entry) -> Option<Result<u64, ParseIntError>> {
        if let Entry::Meta(meta) = meta {
            Some(meta.parse())
        } else {
            None
        }
    }
}

impl<'a, T> Iterator for RunningQueryParser<T>
where
    T: Iterator<Item = Entry<'a>>,
{
    type Item = Result<RunningQueryTable<'a>, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut meta = None;
        let mut table = None;
        while let Some(entry) = self.0.next() {
            if let Entry::Meta(_) = entry {
                meta = Some(entry);
                break;
            }
        }

        while let Some(entry) = self.0.next() {
            if let Entry::Table(_) = entry {
                table = Some(entry);
                break;
            }
        }

        // Some(RunningQueryTable::try_from((meta?, table?)))
        todo!()
    }
}
