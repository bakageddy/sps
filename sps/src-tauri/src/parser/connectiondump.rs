use crate::{parser::tokenizer, util};
use duckdb::types::{FromSql, FromSqlError};
use error::Error;
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use std::{
    borrow::Cow,
    str::{FromStr, Utf8Error},
};

use crate::parser::tokenizer::Tokenizer;
use time::{OffsetDateTime, format_description::BorrowedFormatItem, macros::format_description};

const CONNECTION_DUMP_TIME_FORMAT: &[BorrowedFormatItem] =
    format_description!("[hour]:[minute]:[second].[subsecond]");
const CONNECTION_DUMP_DATE_FORMAT: &[BorrowedFormatItem] =
    format_description!("[day]-[month]-[year]");
const TRACE_DATE_TIME_FORMAT: &[BorrowedFormatItem] = format_description!(
    "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond] [offset_hour sign:mandatory][offset_minute]"
);

#[derive(Debug)]
pub struct ConnectionDumpParser<'a>(&'a str, ParserState);
#[derive(Debug)]
pub enum ParserState {
    Initial,
    Entry,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Signal {
    pub timestamp: u64,
    pub tid: u64,
    pub cause: Cause,
    pub suppressed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stats {
    pub timestamp: u64,
    pub tid: u64,
    pub used: u64,
    pub free: u64,
    pub total: u64,
}

#[derive(Debug, Serialize)]
pub struct TraceDump<'a> {
    pub tid: u64,
    pub timestamp: u64,
    pub traces: Vec<Trace<'a>>,
}

#[derive(Debug, Serialize)]
pub enum ConnectionDumpEntry<'a> {
    Signal(Signal),
    Stats(Stats),
    Trace(TraceDump<'a>),
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trace<'a> {
    #[serde(deserialize_with = "to_millis")]
    pub duration: u64,
    #[serde(deserialize_with = "to_epoch_millis")]
    pub start_time: u64,
    #[serde(borrow)]
    pub stack_trace: Vec<Cow<'a, str>>,
    #[serde_as(as = "DisplayFromStr")]
    pub id: u64,
    #[serde(borrow)]
    pub invoked_by: Option<Cow<'a, str>>,
    #[serde(borrow)]
    pub thread_name: Cow<'a, str>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Cause {
    #[serde(rename = "High CPU")]
    HighCPU,
    #[serde(rename = "High Memory")]
    HighMemory,
    #[serde(rename = "No ManagedConnections")]
    NoManagedConnections,
    #[serde(rename = "URL invocation")]
    URL,
}

impl<'a> ConnectionDumpParser<'a> {
    const CAUSE_PREAMBLE: &'static str = "Going to dump performance logs.";
    const SKIPPING_CAUSE_PREAMBLE: &'static str = "Skipping to dump performance logs";
    const CNX_STATS_PREAMBLE: &'static str = "Connection Pool Stats ::";
    const TRACE_INFO_PREAMBLE: &'static str = "In use TraceInfo ::";

    pub fn new(data: &'a str) -> Self {
        Self(data, ParserState::Initial)
    }

    pub fn parse_entry(line: &'a str) -> Result<ConnectionDumpEntry<'a>, Error> {
        let mut tok = Tokenizer::new(line);
        let time = tok.take_within("[", "]")?;
        let date = tok.take_within("[", "]")?;
        let timestamp = util::unix_timestamp_millis(
            time,
            date,
            CONNECTION_DUMP_TIME_FORMAT,
            CONNECTION_DUMP_DATE_FORMAT,
        )?;
        let _ = tok.take_within("[", "]")?;
        let _ = tok.take_within("[", "]")?;
        let tid = tok.take_within("[", "]")?.parse()?;
        let _ = tok.take_until("::").ok_or_else(|| {
            Error::InvalidFormat(tokenizer::error::Error::DelimiterNotFound("::".to_owned()))
        })?;
        tok.skip_whitespace();
        if tok.peek(Self::CAUSE_PREAMBLE) {
            tok.expect(Self::CAUSE_PREAMBLE)?;
            tok.skip_whitespace();
            tok.expect("Cause")?;
            tok.skip_whitespace();
            tok.expect(":")?;
            tok.skip_whitespace();
            let cause = tok.remaining().trim().parse()?;
            Ok(ConnectionDumpEntry::Signal(Signal {
                cause,
                timestamp,
                tid,
                suppressed: false,
            }))
        } else if tok.peek(Self::SKIPPING_CAUSE_PREAMBLE) {
            tok.expect(Self::SKIPPING_CAUSE_PREAMBLE)?;
            tok.take_until(".").ok_or_else(|| {
                Error::InvalidFormat(tokenizer::error::Error::DelimiterNotFound(".".to_owned()))
            })?;
            tok.take_until(":").ok_or_else(|| {
                Error::InvalidFormat(tokenizer::error::Error::DelimiterNotFound(".".to_owned()))
            })?;
            tok.skip_whitespace();
            let cause = tok.remaining().trim().parse()?;
            Ok(ConnectionDumpEntry::Signal(Signal {
                cause,
                timestamp,
                tid,
                suppressed: true,
            }))
        } else if tok.peek(Self::CNX_STATS_PREAMBLE) {
            tok.expect(Self::CNX_STATS_PREAMBLE)?;

            tok.skip_whitespace();
            tok.expect("used_connections:")?;
            let used = tok
                .take_until(",")
                .ok_or_else(|| {
                    Error::InvalidFormat(tokenizer::error::Error::DelimiterNotFound(",".to_owned()))
                })?
                .trim()
                .parse()?;

            tok.skip_whitespace();
            tok.expect("free_connections:")?;
            let free = tok
                .take_until(",")
                .ok_or_else(|| {
                    Error::InvalidFormat(tokenizer::error::Error::DelimiterNotFound(",".to_owned()))
                })?
                .trim()
                .parse()?;

            tok.skip_whitespace();
            tok.expect("max_connections:")?;
            let total = tok.remaining().trim().parse()?;
            Ok(ConnectionDumpEntry::Stats(Stats {
                tid,
                timestamp,
                used,
                free,
                total,
            }))
        } else if tok.peek(Self::TRACE_INFO_PREAMBLE) {
            tok.expect(Self::TRACE_INFO_PREAMBLE)?;
            tok.skip_whitespace();
            let json = tok.remaining();
            let traces: Vec<Trace> = serde_json::from_str(json)?;
            Ok(ConnectionDumpEntry::Trace(TraceDump {
                tid,
                timestamp,
                traces,
            }))
        } else {
            Err(Error::UnrecognizedConnectionDumpEntry(
                tok.remaining().to_owned(),
            ))
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for ConnectionDumpParser<'a> {
    type Error = Utf8Error;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let result = std::str::from_utf8(value)?;
        Ok(Self::new(result))
    }
}

impl<'a> Iterator for ConnectionDumpParser<'a> {
    type Item = Result<ConnectionDumpEntry<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut tok = Tokenizer::new(self.0);
        tok.skip_whitespace();
        if tok.is_empty() {
            return None;
        }

        while let Some(line) = tok.peek_line() {
            if line.trim_start().starts_with("[") {
                self.1 = ParserState::Entry;
                break;
            }
            tok.get_line()?;
        }

        if let ParserState::Initial = self.1 {
            return None;
        }

        let line = tok.get_line()?;
        match Self::parse_entry(line) {
            Ok(e) => {
                self.0 = tok.remaining();
                Some(Ok(e))
            }
            Err(e) => {
                self.0 = tok.remaining();
                Some(Err(e))
            }
        }
    }
}

impl FromStr for Cause {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "High CPU" => Ok(Cause::HighCPU),
            "No ManagedConnections" => Ok(Cause::NoManagedConnections),
            "High Memory Consumption" => Ok(Cause::HighMemory),
            "URL invocation" => Ok(Cause::URL),
            _ => Err(Error::Cause(s.to_string())),
        }
    }
}

impl TryFrom<&[u8]> for Cause {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            b"High CPU" => Ok(Cause::HighCPU),
            b"No ManagedConnections" => Ok(Cause::NoManagedConnections),
            b"High Memory Consumption" => Ok(Cause::HighMemory),
            b"URL invocation" => Ok(Cause::URL),
            _ => Err(Error::Cause(
                std::str::from_utf8(value)
                    .unwrap_or("Unable to convert bytes to String")
                    .to_string(),
            )),
        }
    }
}

impl FromSql for Cause {
    fn column_result(value: duckdb::types::ValueRef<'_>) -> duckdb::types::FromSqlResult<Self> {
        match value {
            duckdb::types::ValueRef::Text(items) => {
                Cause::try_from(items).map_err(|e| FromSqlError::Other(Box::new(e)))
            }
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl Cause {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::HighCPU => "High CPU",
            Self::NoManagedConnections => "No ManagedConnections",
            Self::HighMemory => "High Memory Consumption",
            Self::URL => "URL Invocation",
        }
    }
}

pub fn to_epoch_millis<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    let dt = OffsetDateTime::parse(s, TRACE_DATE_TIME_FORMAT)
        .map_err(|e| serde::de::Error::custom(e))?;
    Ok((dt.unix_timestamp_nanos() / 1_000_000) as u64)
}

pub fn to_millis<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    let s = s.split_once(" ").and_then(|(n, _)| n.parse().ok());
    Ok(s.unwrap_or(0))
}

pub mod error {
    use crate::{parser::tokenizer, types::TimestampError};
    use std::num::ParseIntError;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Invalid Format: {0}")]
        InvalidFormat(#[from] tokenizer::error::Error),
        #[error("Timestamp parsing: {0}")]
        Timestamp(#[from] TimestampError),
        #[error("Error during parsing number: {0}")]
        Number(#[from] ParseIntError),
        #[error("Invalid Cause: {0}")]
        Cause(String),
        #[error("Unrecognized Connection Dump Entry: {0}")]
        UnrecognizedConnectionDumpEntry(String),
        #[error("Invalid JSON: {0}")]
        JSON(#[from] serde_json::Error),
    }
}

#[cfg(test)]
pub mod test {
    use super::ConnectionDumpParser;
    use crate::{
        parser::connectiondump::{Cause, ConnectionDumpEntry, Signal, Stats, TraceDump},
        util,
    };
    use std::{assert_matches, ops::Deref};

    #[test]
    fn cd_cause_entry() {
        let line = "[14:30:10.237]|[11-02-2026]|[ConnectionDump]|[INFO]|[201]| :: Going to dump performance logs. Cause : High CPU ";
        let entry = ConnectionDumpParser::parse_entry(line);
        assert!(
            entry.is_ok(),
            "Error during parsing: {}",
            entry.unwrap_err()
        );
        let entry = entry.unwrap();
        assert_matches!(entry, ConnectionDumpEntry::Signal(_));
        if let ConnectionDumpEntry::Signal(Signal {
            cause,
            timestamp,
            tid,
            suppressed,
        }) = entry
        {
            assert_matches!(cause, Cause::HighCPU);
            assert_ne!(timestamp, 0);
            assert_eq!(tid, 201);
            assert_eq!(suppressed, false);
        }
    }

    #[test]
    fn cd_connection_pool_stats() {
        let line = "[14:30:11.022]|[11-02-2026]|[ConnectionDump]|[INFO]|[205]| :: Connection Pool Stats :: used_connections:9, free_connections:71, max_connections:80";
        let entry = ConnectionDumpParser::parse_entry(line);
        assert!(
            entry.is_ok(),
            "Error during parsing: {}",
            entry.unwrap_err()
        );
        let entry = entry.unwrap();
        assert_matches!(entry, ConnectionDumpEntry::Stats(_));
        if let ConnectionDumpEntry::Stats(Stats {
            tid,
            timestamp,
            used,
            free,
            total,
        }) = entry
        {
            assert_eq!(tid, 205);
            assert_ne!(timestamp, 0);
            assert_eq!(used, 9);
            assert_eq!(free, 71);
            assert_eq!(total, 80);
        }

        let map = util::map_file("test/cd0/cd1_traces.txt").unwrap();
        let data = std::str::from_utf8(map.deref()).unwrap();
        let entry = ConnectionDumpParser::parse_entry(data);
        assert!(
            entry.is_ok(),
            "Error during parsing: {}",
            entry.unwrap_err()
        );
        let entry = entry.unwrap();
        if let ConnectionDumpEntry::Trace(TraceDump {
            tid,
            timestamp,
            traces,
        }) = entry
        {
            assert_eq!(tid, 22388);
            assert_ne!(timestamp, 0);
            assert_eq!(traces.len(), 38);
        }
    }

    #[test]
    fn cd_connection_pool_traces() {
        let map = util::map_file("test/cd0/cd0_traces.txt").unwrap();
        let data = std::str::from_utf8(map.deref()).unwrap();
        let entry = ConnectionDumpParser::parse_entry(data);
        assert!(
            entry.is_ok(),
            "Error during parsing: {}",
            entry.unwrap_err()
        );
        let entry = entry.unwrap();
        if let ConnectionDumpEntry::Trace(TraceDump {
            tid,
            timestamp,
            traces,
        }) = entry
        {
            assert_eq!(tid, 205);
            assert_ne!(timestamp, 0);
            assert_eq!(traces.len(), 9);
        }
    }

    #[test]
    fn cd_skipping_performance() {
        let line = "[15:08:03.660]|[11-02-2026]|[ConnectionDump]|[INFO]|[6744]| :: Skipping to dump performance logs for 960 seconds from last dump. Current trigger- Cause : High CPU";
        let entry = ConnectionDumpParser::parse_entry(line);
        assert!(
            entry.is_ok(),
            "Error during parsing: {}",
            entry.unwrap_err()
        );
        let entry = entry.unwrap();
        if let ConnectionDumpEntry::Signal(Signal {
            cause,
            timestamp,
            tid,
            suppressed,
        }) = entry
        {
            assert_ne!(timestamp, 0);
            assert_eq!(tid, 6744);
            assert_matches!(cause, Cause::HighCPU);
            assert_eq!(suppressed, true);
        }
    }

    #[test]
    fn cd_skipping_performance_nmc() {
        let line = "[02:04:00.888]|[29-07-2026]|[ConnectionDump]|[INFO]|[67]| :: Skipping to dump performance logs for 120 seconds from last dump. Current trigger- Cause : No ManagedConnections";
        let entry = ConnectionDumpParser::parse_entry(line);
        assert!(
            entry.is_ok(),
            "Error during parsing: {}",
            entry.unwrap_err()
        );
        let entry = entry.unwrap();
        if let ConnectionDumpEntry::Signal(Signal {
            cause,
            timestamp,
            tid,
            suppressed,
        }) = entry
        {
            assert_ne!(timestamp, 0);
            assert_eq!(tid, 67);
            assert_matches!(cause, Cause::NoManagedConnections);
            assert_eq!(suppressed, true);
        }
    }

    #[test]
    fn cd_full_file() {
        let map = util::map_file("test/cd0/cd0.txt").unwrap();
        let parser = ConnectionDumpParser::try_from(map.deref()).unwrap();
        let mut count = 0;
        for entry in parser {
            // assert!(entry.is_ok(), "Error during parsing: {}", entry.unwrap_err());
            if entry.is_ok() {
                count += 1;
            } else {
                dbg!(entry.unwrap_err());
            }
        }
        assert_eq!(count, 67);

        let map = util::map_file("test/cd0/cd1.txt").unwrap();
        let parser = ConnectionDumpParser::try_from(map.deref()).unwrap();
        let mut count = 0;
        for entry in parser {
            // assert!(entry.is_ok(), "Error during parsing: {}", entry.unwrap_err());
            if entry.is_ok() {
                count += 1;
            } else {
                dbg!(entry.unwrap_err());
            }
        }
        assert_eq!(count, 422);
    }
}
