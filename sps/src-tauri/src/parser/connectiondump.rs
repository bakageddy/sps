use crate::{parser::tokenizer, util};
use error::Error;
use serde::Deserialize;
use std::{
    borrow::Cow,
    str::{FromStr, Utf8Error},
};

use crate::parser::tokenizer::Tokenizer;
use time::{format_description::BorrowedFormatItem, macros::format_description};

const CONNECTION_DUMP_TIME_FORMAT: &[BorrowedFormatItem] =
    format_description!("[hour]:[minute]:[second].[subsecond]");
const CONNECTION_DUMP_DATE_FORMAT: &[BorrowedFormatItem] =
    format_description!("[day]-[month]-[year]");

pub struct ConnectionDumpParser<'a>(&'a str, ParserState);
pub enum ParserState {
    Initial,
    Entry,
}

pub enum ConnectionDumpEntry<'a> {
    Signal {
        cause: Cause,
        timestamp: u64,
        tid: u64,
    },
    ConnectionPoolStats {
        tid: u64,
        timestamp: u64,
        used: u64,
        free: u64,
        total: u64,
    },
    ConnectionPoolTrace {
        tid: u64,
        timestamp: u64,
        traces: Vec<Trace<'a>>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trace<'a> {
    duration: u64,
    #[serde(with = "")]
    start_time: u64,
    stacktrace: Vec<Cow<'a, str>>,
    id: u64,
    invoked_by: Cow<'a, str>,
    thread_name: Cow<'a, str>,
}

pub enum Cause {
    HighCPU,
    HighMemory,
    NoManagedConnections,
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
            Ok(ConnectionDumpEntry::Signal {
                cause,
                timestamp,
                tid,
            })
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
            Ok(ConnectionDumpEntry::Signal {
                cause,
                timestamp,
                tid,
            })
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
            let free = tok
                .take_until(",")
                .ok_or_else(|| {
                    Error::InvalidFormat(tokenizer::error::Error::DelimiterNotFound(",".to_owned()))
                })?
                .trim()
                .parse()?;
            let total = tok.remaining().trim().parse()?;
            Ok(ConnectionDumpEntry::ConnectionPoolStats {
                tid,
                timestamp,
                used,
                free,
                total,
            })
        } else if tok.peek(Self::TRACE_INFO_PREAMBLE) {
            tok.expect(Self::TRACE_INFO_PREAMBLE)?;
            tok.skip_whitespace();
            let json = tok.remaining();
            serde_json::from_str(json);
            todo!()
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
        let entry = match Self::parse_entry(line) {
            Ok(e) => e,
            Err(e) => {
                self.0 = tok.remaining();
                return Some(Err(e));
            }
        };
        Some(Ok(entry))
    }
}

impl FromStr for Cause {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "High CPU" => Ok(Cause::HighCPU),
            "No ManagedConnections" => Ok(Cause::NoManagedConnections),
            "High Memory" => Ok(Cause::HighMemory),
            "URL invocation" => Ok(Cause::URL),
            _ => Err(Error::Cause(s.to_string())),
        }
    }
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
    }
}
