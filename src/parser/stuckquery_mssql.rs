use std::str::FromStr;

use time::{Date, PrimitiveDateTime, Time, macros::format_description};
use tracing::warn;

use crate::{
    error::stuckquery::SqlServerParse,
    parser::{
        scanner::Scanner,
        stuckquery_pgsql::{TIMESTAMP_DATE_FORMAT, TIMESTAMP_TIME_FORMAT},
    },
    util::ToUnixMillis,
};

static LAST_REQUEST_FORMAT: &[time::format_description::FormatItem<'static>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]");

#[derive(Debug)]
pub struct StuckQueryTable<'a> {
    pub queries: Vec<StuckQuery<'a>>,
    pub timestamp: u64,
}

#[derive(Debug)]
pub struct StuckQuery<'a> {
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
    pub login_time: u64,
    pub open_transaction_count: u64,
}

#[derive(Debug)]
pub enum Status {
    Running,
    Runnable,
    Suspended,
}

impl FromStr for Status {
    type Err = SqlServerParse;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "running" => Ok(Self::Running),
            "runnable" => Ok(Self::Runnable),
            "suspended" => Ok(Self::Suspended),
            s => Err(SqlServerParse::InvalidStatus(String::from(s))),
        }
    }
}

impl<'a> StuckQueryTable<'a> {
    fn extract_timestamp(header: &str) -> Result<u64, SqlServerParse> {
        let mut scanner = Scanner::new(header);
        let time = scanner
            .take_within("[", "]")
            .map_err(|_| SqlServerParse::TableHeaderMetaExtraction)?;
        let date = scanner
            .take_within("[", "]")
            .map_err(|_| SqlServerParse::TableHeaderMetaExtraction)?;
        let parsed_time =
            Time::parse(time, TIMESTAMP_TIME_FORMAT).map_err(SqlServerParse::TimestampParse)?;
        let parsed_date =
            Date::parse(date, TIMESTAMP_DATE_FORMAT).map_err(SqlServerParse::TimestampParse)?;
        let datetime = PrimitiveDateTime::new(parsed_date, parsed_time);
        Ok(datetime.to_unix_millis().unwrap_or(0))
    }
}

impl<'a> TryFrom<&'a str> for StuckQueryTable<'a> {
    type Error = SqlServerParse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut queries = Vec::new();
        let mut scanner = Scanner::new(value);
        let header = scanner.take_until("\n");
        let timestamp = header
            .ok_or(SqlServerParse::TableHeaderExtraction)
            .map(Self::extract_timestamp)??;

        for _ in 0..7 {
            if scanner.take_until("\n").is_none() {
                return Err(SqlServerParse::TableExtraction);
            }
        }

        while let Some(line) = scanner.take_until("\n") {
            if !line.trim().starts_with("|") {
                break;
            }

            match StuckQuery::try_from(line) {
                Ok(x) => queries.push(x),
                Err(e) => warn!("Cannot parse {line} due to {e:?}"),
            }
        }
        Ok(Self { timestamp, queries })
    }
}

impl<'a> TryFrom<&'a str> for StuckQuery<'a> {
    type Error = SqlServerParse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut scanner = value.split("|");
        scanner.next().ok_or_else(|| SqlServerParse::EmptyBlock)?;
        let session_id = scanner
            .next()
            .ok_or_else(|| SqlServerParse::SessionIDExtraction)?
            .parse::<u64>()
            .map_err(SqlServerParse::SessionIDParse)?;

        let status = scanner
            .next()
            .ok_or_else(|| SqlServerParse::StatusExtraction)?
            .parse::<Status>()?;

        let txn_id = scanner
            .next()
            .ok_or_else(|| SqlServerParse::TransactionIDExtraction)?
            .parse::<u64>()
            .map_err(SqlServerParse::TransactionIDParse)?;

        let blocked_by = scanner
            .next()
            .ok_or_else(|| SqlServerParse::BlockedByExtraction)?
            .parse::<u64>()
            .map_err(SqlServerParse::BlockedByParse)?;

        let wait_type = scanner
            .next()
            .ok_or_else(|| SqlServerParse::WaitTypeExtraction)?
            .trim();
        let wait_type = if wait_type.is_empty() {
            None
        } else {
            Some(wait_type)
        };

        let wait_resource = scanner
            .next()
            .ok_or_else(|| SqlServerParse::WaitResourceExtraction)?
            .trim();
        let wait_resource = if wait_resource.is_empty() {
            None
        } else {
            Some(wait_resource)
        };

        let wait_time = scanner
            .next()
            .ok_or_else(|| SqlServerParse::WaitTimeExtraction)?
            .parse::<f32>()
            .map_err(SqlServerParse::WaitTimeParse)?;

        let wait_time_ms = (wait_time * 1000.0f32).trunc() as u64;

        let cpu_time = scanner
            .next()
            .ok_or_else(|| SqlServerParse::CPUTimeExtraction)?
            .parse::<f32>()
            .map_err(SqlServerParse::CPUTimeParse)?;

        let cpu_time_ms = (cpu_time * 1000.0f32).trunc() as u64;

        let logical_reads = scanner
            .next()
            .ok_or_else(|| SqlServerParse::LogicalReadsExtraction)?
            .parse::<u64>()
            .map_err(SqlServerParse::LogicalReadsParse)?;

        let physical_reads = scanner
            .next()
            .ok_or_else(|| SqlServerParse::PhysicalReadsExtraction)?
            .parse::<u64>()
            .map_err(SqlServerParse::PhysicalReadsParse)?;

        let elapsed_time = scanner
            .next()
            .ok_or_else(|| SqlServerParse::ElapsedTimeExtraction)?
            .parse::<f32>()
            .map_err(SqlServerParse::ElapsedTimeParse)?;

        let elapsed_time_ms = (elapsed_time * 1000.0f32).trunc() as u64;

        let statement = scanner
            .next()
            .ok_or_else(|| SqlServerParse::StatementExtraction)?
            .trim();

        let command_text = scanner
            .next()
            .ok_or_else(|| SqlServerParse::CommandTextExtraction)?
            .trim();

        let command_text = if command_text.is_empty() {
            None
        } else {
            Some(command_text)
        };

        let command = scanner
            .next()
            .ok_or_else(|| SqlServerParse::CommandExtraction)?
            .trim();

        let command = if command.is_empty() {
            None
        } else {
            Some(command)
        };

        let login_name = scanner
            .next()
            .ok_or_else(|| SqlServerParse::LoginNameExtraction)?
            .trim();

        let host_name = scanner
            .next()
            .ok_or_else(|| SqlServerParse::HostNameExtraction)?
            .trim();

        let db_name = scanner
            .next()
            .ok_or_else(|| SqlServerParse::DatabaseNameExtraction)?
            .trim();

        let program_name = scanner
            .next()
            .ok_or_else(|| SqlServerParse::ProgramNameExtraction)?
            .trim();

        let host_process_id = scanner
            .next()
            .ok_or_else(|| SqlServerParse::HostProcessIDExtraction)?
            .parse::<u64>()
            .map_err(SqlServerParse::HostProcessIDParse)?;

        let last_request_end_ms = scanner
            .next()
            .ok_or_else(|| SqlServerParse::LastRequestEndExtraction)?
            .trim();

        let last_request_end_ms =
            PrimitiveDateTime::parse(last_request_end_ms, LAST_REQUEST_FORMAT)
                .map_err(SqlServerParse::LastRequestEndParse)?
                .to_unix_millis()
                .unwrap_or(0);

        let login_time_ms = scanner
            .next()
            .ok_or_else(|| SqlServerParse::LoginTimeExtraction)?
            .trim();

        let login_time_ms = PrimitiveDateTime::parse(login_time_ms, LAST_REQUEST_FORMAT)
            .map_err(SqlServerParse::LoginTimeParse)?
            .to_unix_millis()
            .unwrap_or(0);

        let open_transaction_count = scanner
            .next()
            .ok_or_else(|| SqlServerParse::OpenTransactionCountExtraction)?
            .parse::<u64>()
            .map_err(SqlServerParse::OpenTransactionCountParse)?;

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
            login_time,
            open_transaction_count,
        })
    }
}
