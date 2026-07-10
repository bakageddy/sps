use std::{net::Ipv4Addr, num::ParseIntError};

use time::{UtcDateTime, macros::format_description};

use crate::{
    error::running_query::{Error, PGParse},
    ingest::running_queries::Entry,
    parser::scanner::Scanner,
};

static PGSQL_STATE_CHANGE_FORMAT: &[time::format_description::FormatItem<'static>] = format_description!(
    "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond][offset_hour sign:mandatory]:[offset_minute]"
);

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
    RunningQueries(MSSQLRunningQueries<'a>),
}

#[derive(Debug)]
pub struct SPWho2Table<'a> {
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
pub struct MSSQLRunningQueries<'a> {
    pub session_id: u64,
    pub status: Status,
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
pub enum Status {
    Running,
    Runnable,
    Suspended,
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

        Some(RunningQueryTable::try_from((meta?, table?)))
    }
}

impl<'a> TryFrom<(Entry<'a>, Entry<'a>)> for RunningQueryTable<'a> {
    type Error = Error;

    fn try_from(value: (Entry<'a>, Entry<'a>)) -> Result<Self, Self::Error> {
        let timestamp = Self::extract_timestamp(value.0);
        todo!()
    }
}
