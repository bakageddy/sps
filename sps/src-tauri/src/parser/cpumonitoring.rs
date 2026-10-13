use std::borrow::Cow;

use time::format_description::BorrowedFormatItem;
use time::macros::format_description;
use time_tz::Tz;
use tracing::warn;

use crate::parser::Context;
use crate::parser::cpumonitoring::error::Error;
use crate::parser::tokenizer::{Parser, Tokenizer};
use crate::util;

const CPU_MONITORING_TIME_FORMAT: &[BorrowedFormatItem] =
    format_description!("[hour]:[minute]:[second].[subsecond]");
const CPU_MONITORING_DATE_FORMAT: &[BorrowedFormatItem] =
    format_description!("[day]-[month]-[year]");

#[derive(Debug)]
pub struct CPUMonitoringParser<'a>(&'a str, ParserState, Context);

// FIXME: when we return Some(Err(e)), call tok.remaining()
impl<'a> Iterator for CPUMonitoringParser<'a> {
    type Item = Result<CPUMonitoring<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0 = self.0.trim_start();
        if self.0.is_empty() {
            return None;
        }
        let mut tok = Tokenizer::new(self.0);
        let mut utc = None;

        while let Some(line) = tok.get_line() {
            if line.starts_with("[") {
                if let Some(next) = tok.peek_line()
                    && next.starts_with("Thread")
                {
                    utc = Some(line);
                    self.1 = ParserState::EntryHeader;
                    break;
                } else {
                    continue;
                }
            }
        }

        if !matches!(self.1, ParserState::EntryHeader) {
            self.0 = tok.remaining();
            return None;
        }

        let utc = if let Some(line) = utc {
            let t = match Self::parse_timestamp(line, self.2.zone) {
                Ok(t) => t,
                Err(e) => {
                    self.0 = tok.remaining();
                    return Some(Err(e));
                }
            };
            t
        } else {
            self.0 = tok.remaining();
            return None;
        };

        let (tid, cpu, name, state) = match Self::parse_header(&mut tok) {
            Ok(x) => x,
            Err(e) => {
                self.0 = tok.remaining();
                return Some(Err(e));
            }
        };

        self.1 = ParserState::EntryTrace;

        let mut frames = Vec::new();
        while let Some(line) = tok.peek_line()
            && !line.trim().is_empty()
            && line.trim_start().starts_with("at")
        {
            let frame = Frame::parse(line);
            if let Err(e) = frame {
                return Some(Err(e));
            }
            frames.push(frame.unwrap());
            tok.get_line();
        }

        let trace = if frames.is_empty() {
            None
        } else {
            Some(CPUTrace(frames))
        };

        self.0 = tok.remaining();
        Some(Ok(CPUMonitoring {
            timestamp: utc,
            tid,
            usage: cpu,
            name,
            state,
            trace,
        }))
    }
}

impl<'a> CPUMonitoringParser<'a> {
    pub fn new(data: &'a str) -> Self {
        Self(
            data,
            ParserState::Initial,
            Context {
                zone: None,
                handle: None,
            },
        )
    }

    pub fn with_context(data: &'a str, ctx: Context) -> Self {
        Self(data, ParserState::Initial, ctx)
    }

    pub fn parse_timestamp(data: &'a str, zone: Option<&'static Tz>) -> Result<u64, Error> {
        let mut tok = Tokenizer::new(data);
        let time = tok.take_within("[", "]")?;
        let date = tok.take_within("[", "]")?;
        let timestamp = util::unix_timestamp_millis(
            time,
            date,
            CPU_MONITORING_TIME_FORMAT,
            CPU_MONITORING_DATE_FORMAT,
            zone,
        )?;
        Ok(timestamp)
    }

    pub fn parse_header<'b>(
        tok: &mut Tokenizer<'b>,
    ) -> Result<(u64, f32, Option<Cow<'b, str>>, State), Error> {
        tok.expect("Thread Info:")?;
        tok.expect("Thread Id :")?;
        tok.skip_whitespace();
        let tid = tok.take_until_fallible(",")?.parse()?;
        tok.skip_whitespace();
        tok.expect("has CPU usage :")?;
        let cpu = tok.take_until_fallible("\n")?.trim().parse()?;
        tok.expect("Thread Name:")?;
        let name = tok.take_until_fallible(",")?.trim();
        let name = if name.is_empty() {
            None
        } else {
            Some(name.into())
        };

        tok.skip_whitespace();
        tok.expect("Thread State:")?;
        let state = tok.take_until_fallible("\n")?.trim();
        let state = State::from(state);

        Ok((tid, cpu, name, state))
    }
}

impl<'a> TryFrom<&'a [u8]> for CPUMonitoringParser<'a> {
    type Error = std::str::Utf8Error;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let value = std::str::from_utf8(value)?;
        Ok(Self::new(value))
    }
}

#[derive(Debug)]
enum ParserState {
    Initial,
    EntryHeader,
    EntryTrace,
}

#[derive(Debug)]
pub struct CPUMonitoring<'a> {
    pub timestamp: u64,
    pub tid: u64,
    pub usage: f32,
    pub name: Option<Cow<'a, str>>,
    pub state: State,
    pub trace: Option<CPUTrace<'a>>,
}

#[derive(Debug)]
pub struct CPUTrace<'a>(pub Vec<Frame<'a>>);

impl<'a> Parser<'a> for CPUTrace<'a> {
    type Error = Error;
    type Context = ();

    fn parse_with(data: &'a str, _ctx: &Self::Context) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut tok = Tokenizer::new(data);
        let mut frames = Vec::new();
        while let Some(line) = tok.get_line() {
            let frame = Frame::parse(line)?;
            frames.push(frame);
        }
        Ok(CPUTrace(frames))
    }
}

#[derive(Debug)]
pub struct Frame<'a> {
    pub method: Cow<'a, str>,
    pub source: Cow<'a, str>,
}

impl<'a> Parser<'a> for Frame<'a> {
    type Error = Error;
    type Context = ();

    /// Expect
    /// <whitespace>at<whitespace><method>(<source>)\n
    fn parse_with(data: &'a str, _ctx: &Self::Context) -> Result<Self, Self::Error> {
        let mut tok = Tokenizer::new(data);
        tok.skip_whitespace();
        tok.expect("at").map_err(Error::InvalidFormat)?;
        tok.skip_whitespace();
        let method = tok
            .take_until_exclusive("(")
            .map_err(Error::InvalidFormat)?
            .into();
        let source = tok
            .take_within("(", ")")
            .map_err(Error::InvalidFormat)?
            .into();
        Ok(Frame { method, source })
    }
}

#[derive(Debug, PartialEq)]
pub enum State {
    New,
    Runnable,
    Blocked,
    Waiting,
    TimedWaiting,
    Terminated,
    Unknown,
}

impl From<&str> for State {
    fn from(value: &str) -> Self {
        match value {
            "NEW" => Self::New,
            "RUNNABLE" => Self::Runnable,
            "BLOCKED" => Self::Blocked,
            "WAITING" => Self::Waiting,
            "TIMED_WAITING" => Self::TimedWaiting,
            "TERMINATED" => Self::Terminated,
            e => {
                warn!("Unknow state: {e}");
                Self::Unknown
            }
        }
    }
}

impl State {
    pub fn into_str(&self) -> &'static str {
        match self {
            Self::New => "NEW",
            Self::Runnable => "RUNNABLE",
            Self::Blocked => "BLOCKED",
            Self::Waiting => "WAITING",
            Self::TimedWaiting => "TIMED_WAITING",
            Self::Terminated => "TERMINATED",
            Self::Unknown => "UNKNOWN",
        }
    }
}

pub mod error {
    use crate::{parser::tokenizer, types::TimestampError};
    #[non_exhaustive]
    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Invalid Format for stack trace/cpu monitoring entry: {0:?}")]
        InvalidFormat(#[from] tokenizer::error::Error),
        #[error("Invalid Format for Timestamp: {0:?}")]
        ParseTimestamp(#[from] TimestampError),
        #[error("Unable to parse Thread ID: {0:?}")]
        ParseThreadID(#[from] std::num::ParseIntError),
        #[error("Unable to parse Thread CPU Usage: {0:?}")]
        ParseCPU(#[from] std::num::ParseFloatError),
    }
}

#[cfg(test)]
mod test {
    use std::ops::Deref;

    use crate::parser::cpumonitoring::{CPUMonitoringParser, State};
    use crate::util::map_file;

    #[test]
    fn cpumonitoring_state_from() {
        let state = "RUNNABLE";
        let parsed = State::from(state);
        assert!(matches!(parsed, State::Runnable));
        let state = "NEW";
        let parsed = State::from(state);
        assert!(matches!(parsed, State::New));
        let state = "TERMINATED";
        let parsed = State::from(state);
        assert!(matches!(parsed, State::Terminated));
        let state = "WAITING";
        let parsed = State::from(state);
        assert!(matches!(parsed, State::Waiting));
        let state = "TIMED_WAITING";
        let parsed = State::from(state);
        assert!(matches!(parsed, State::TimedWaiting));
        let state = "BLOCKED";
        let parsed = State::from(state);
        assert!(matches!(parsed, State::Blocked));
    }

    #[test]
    fn cpumonitoring_entries_no_trace() {
        let map = &map_file("test/CPUMonitoring/header_only.txt").unwrap();
        for entry in CPUMonitoringParser::try_from(map.deref()).unwrap() {
            assert!(
                entry.is_ok(),
                "Error during parsing: {}",
                entry.unwrap_err()
            );
            let entry = entry.unwrap();
            assert_ne!(entry.timestamp, 0);
            assert!(entry.usage <= 0.5);
            assert!(
                entry.trace.is_none(),
                "Error during parsing! Traces not empty"
            );
        }
    }

    #[test]
    fn cpumonitoring_entry_single() {
        let map = map_file("test/CPUMonitoring/single_entry.txt").unwrap();
        let mut iter = CPUMonitoringParser::try_from(map.deref()).unwrap();
        let entry = iter.next();
        assert!(entry.is_some(), "Expect single entry");
        let entry = entry.unwrap();
        assert!(
            entry.is_ok(),
            "Error during parsing: {}",
            entry.unwrap_err()
        );
        let entry = entry.unwrap();
        assert!(entry.trace.is_some());
        assert_eq!(entry.name, Some("Asset_8:XML".into()));
        assert_eq!(entry.usage, 1.7284292);
        assert_eq!(entry.state, State::TimedWaiting);
        assert_eq!(entry.tid, 5227u64);
    }

    #[test]
    fn cpumonitoring_dump_single() {
        let map = map_file("test/CPUMonitoring/single_dump.txt").unwrap();
        let mut iter = CPUMonitoringParser::try_from(map.deref()).unwrap();
        let mut count = 0;
        while let Some(entry) = iter.next() {
            assert!(
                entry.is_ok(),
                "Error during parsing: {}",
                entry.unwrap_err()
            );
            count += 1;
        }

        assert_eq!(count, 191);
    }

    #[test]
    fn cpumonitoring_file_single() {
        let map = map_file("test/CPUMonitoring/single_file.txt").unwrap();
        let mut iter = CPUMonitoringParser::try_from(map.deref()).unwrap();
        let mut count = 0;
        while let Some(entry) = iter.next() {
            assert!(
                entry.is_ok(),
                "Error during parsing: {}",
                entry.unwrap_err()
            );
            count += 1;
        }
        assert_eq!(count, 726);
    }
}
