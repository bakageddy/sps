use std::{iter::Peekable, net::Ipv4Addr, num::ParseIntError};

use time::{Date, PrimitiveDateTime, Time, UtcDateTime, macros::format_description};
use tracing::warn;

use crate::{
    error::running_query::{Error, MSParse, PGParse},
    ingest::running_queries::Entry,
    parser::{scanner::Scanner, stuckquery::Kind},
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
pub struct PGSQLQuery<'a> {
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
    pub queries: Vec<PGSQLQuery<'a>>,
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
    pub entries: Vec<SPWho2Entry<'a>>,
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

impl<'a> From<&'a str> for SPWho2Status<'a> {
    fn from(value: &'a str) -> Self {
        match value {
            "sleeping" => Self::Sleeping,
            "BACKGROUND" => Self::Background,
            "RUNNABLE" => Self::Runnable,
            unknown => Self::Unknown(unknown),
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

        let timestamp = Self::extract_timestamp(meta)
            .map_err(|e| warn!("Error during parsing timestamp due to {}", e))
            .unwrap_or(0);
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

            match PGSQLQuery::try_from(line) {
                Ok(x) => queries.push(x),
                Err(e) => warn!("Cannot parse {line} due to {e:?}"),
            }
        }
        Ok(Self { timestamp, queries })
    }
}

impl<'a> TryFrom<&'a str> for PGSQLQuery<'a> {
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
            "sp Who2" => {
                for _ in 0..3 {
                    let line = scanner.take_until("\n");
                    if line.is_none() {
                        warn!("Table is empty, skipping parsing");
                        return Ok(Self::SPWho2(SPWho2Table {
                            timestamp,
                            entries: Vec::new(),
                        }));
                    }
                }

                let mut entries = Vec::with_capacity(40);
                while let Some(line) = scanner.take_until("\n") {
                    if !line.trim().starts_with("|") {
                        break;
                    }

                    match SPWho2Entry::try_from(line) {
                        Ok(entry) => entries.push(entry),
                        Err(e) => warn!("Cannot parse spwho2 entry due to {e:?}"),
                    }
                }

                return Ok(Self::SPWho2(SPWho2Table { timestamp, entries }));
            }
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
            .parse()
            .map_err(MSParse::SpWho2ParseSPID)?;

        let status = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::SpWho2ExtractionStatus)?
            .trim();

        let status = SPWho2Status::from(status);

        let login = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::SpWho2ExtractionLogin)?
            .trim();

        let hostname = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::SpWho2ExtractionHostname)?
            .trim();

        let blocked_by = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::SpWho2ExtractionBlockedBy)?
            .trim();

        let blocked_by = if blocked_by == "." {
            None
        } else {
            blocked_by.parse().ok()
        };

        let db_name = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::SpWho2ExtractionDBName)?
            .trim();

        let command = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::SpWho2ExtractionCommand)?
            .trim();

        let cpu_time = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::SpWho2ExtractionCPUTime)?
            .trim()
            .parse()
            .map_err(MSParse::SpWho2ParseCPUTime)?;

        let disk_io = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::SpWho2ExtractionDiskIO)?
            .trim()
            .parse()
            .map_err(MSParse::SpWho2ParseDiskIO)?;

        let last_batch = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::SpWho2ExtractionBatch)?
            .trim();

        let program_name = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::SpWho2ExtractionProgramName)?
            .trim();

        let request_id = scanner
            .take_within_exclusive("|", "|")
            .map_err(MSParse::SpWho2ExtractionRequestID)?
            .trim();

        Ok(Self {
            spid,
            status,
            login,
            hostname,
            blocked_by,
            db_name,
            command,
            cpu_time,
            disk_io,
            last_batch,
            program_name,
            request_id,
        })
    }
}

pub struct RunningQueryParser<'a, T>(pub Peekable<T>, pub Kind)
where
    T: Iterator<Item = Entry<'a>>;

impl<'a, T> RunningQueryParser<'a, T>
where
    T: Iterator<Item = Entry<'a>>,
{
    pub fn new(kind: Kind, iter: T) -> Self
    where
        T: Iterator<Item = Entry<'a>>,
    {
        Self(iter.peekable(), kind)
    }
}

impl<'a, T> Iterator for RunningQueryParser<'a, T>
where
    T: Iterator<Item = Entry<'a>>,
{
    type Item = Result<RunningQueryTable<'a>, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut last_meta = None;
        let mut table = None;
        while let Some(entry) = self.0.next() {
            if !matches!(entry, Entry::Meta(_)) {
                continue;
            }

            last_meta = Some(entry);
            break;
        }

        while let Some(entry) = self.0.next() {
            match entry {
                Entry::Meta(_) => last_meta = Some(entry),
                Entry::Table(_) => table = Some(entry),
                _ => continue,
            }
        }

        if let (Some(meta), Some(table)) = (last_meta, table) {
            Some(RunningQueryTable::try_from((self.1.clone(), meta, table)))
        } else {
            None
        }
    }
}

impl<'a> TryFrom<(Kind, Entry<'a>, Entry<'a>)> for RunningQueryTable<'a> {
    type Error = Error;

    fn try_from(value: (Kind, Entry<'a>, Entry<'a>)) -> Result<Self, Self::Error> {
        let value = match value.0 {
            Kind::PGSQL => Self::PGSQL(PGSQLRunningQueryTable::try_from((value.1, value.2))?),
            Kind::MSSQL => Self::MSSQL(MSSQLRunningQueryTable::try_from((value.1, value.2))?),
        };
        Ok(value)
    }
}

#[cfg(test)]
mod test {
    use std::net::Ipv4Addr;

    use crate::{
        ingest::running_queries::Entry,
        parser::running_queries::{PGSQLQuery, PGSQLRunningQueryTable, State},
        util,
    };
    #[test]
    fn running_query_pgsql_single_line() {
        let query = "|  13852  |  0.022713        |  1.095177      |  servicedesk  |  idle in transaction  |  f        |  INSERT INTO PendingIndexRecords (RECORDID,MODULEID,PKCOLUMNVALUE,OPERATIONID,HELPDESKID) VALUES ($1,$2,$3,$4,$5)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |  2026-04-10 16:54:44.171349+05:30  |  PostgreSQL JDBC Driver  |  127.0.0.1       |                   |  64361        |";
        let result = PGSQLQuery::try_from(query);
        assert!(
            result.is_ok(),
            "Error during parsing: {}",
            result.unwrap_err()
        );
        let query = result.unwrap();
        assert_eq!(query.pid, 13852);
        assert_eq!(query.query_time_ms, 22);
        assert_eq!(query.txn_time_ms, 1095);
        assert_eq!(query.db_name, "servicedesk");
        assert!(matches!(query.state, State::Idle));
        assert_eq!(query.waiting, false);
        assert_eq!(query.application_name, "PostgreSQL JDBC Driver");
        assert_eq!(query.client_address, Some(Ipv4Addr::LOCALHOST));
        assert_eq!(query.client_hostname, None);
        assert_eq!(query.client_port, Some(64361));
    }

    #[test]
    fn running_query_pgsql_table() {
        let map = util::map_file("test/pgsql_running_query_single_table.txt").unwrap();
        let table = str::from_utf8(&map).unwrap();
        let result = PGSQLRunningQueryTable::try_from((Entry::Meta(""), Entry::Table(table)));
        assert!(
            result.is_ok(),
            "Error during parsing table: {}",
            result.unwrap_err()
        );
        let table = result.unwrap();
        assert_eq!(table.queries.len(), 15);
        assert_eq!(table.timestamp, 0);
    }
}
