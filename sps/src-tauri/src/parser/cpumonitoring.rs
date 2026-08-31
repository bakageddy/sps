use time::Date;
use time::PlainDateTime;
use time::Time;
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;
use tracing::warn;

use crate::parser::cpumonitoring::error::Error;
use crate::parser::tokenizer;
use crate::parser::tokenizer::{Parser, Tokenizer};

const CPU_MONITORING_TIME_FORMAT: &[BorrowedFormatItem] =
    format_description!("[hour]:[minute]:[second].[subsecond]");
const CPU_MONITORING_DATE_FORMAT: &[BorrowedFormatItem] =
    format_description!("[day]-[month]-[year]");

#[derive(Debug)]
pub struct CPUMonitoringParser<'a>(&'a str, ParserState);

// TODO: Do not construct separate tokenizers
impl<'a> Iterator for CPUMonitoringParser<'a> {
    type Item = Result<CPUMonitoring<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0 = self.0.trim_start();
        if self.0.is_empty() {
            return None;
        }
        let mut iter = Tokenizer::new(self.0);
        // let mut iter = self.0.split_inclusive('\n').peekable();

        let mut utc = None;

        while let Some(line) = iter.get_line() {
            if line.starts_with("[") {
                if let Some(next) = iter.peek_line()
                    && next.starts_with("Thread")
                {
                    let mut header_tokenizer = Tokenizer::new(line);
                    let time = match header_tokenizer
                        .take_within("[", "]")
                        .map_err(Error::InvalidFormat)
                    {
                        Ok(time) => time,
                        Err(e) => return Some(Err(e)),
                    };
                    let date = match header_tokenizer
                        .take_within("[", "]")
                        .map_err(Error::InvalidFormat)
                    {
                        Ok(date) => date,
                        Err(e) => return Some(Err(e)),
                    };

                    let parsed_time = match Time::parse(time, CPU_MONITORING_TIME_FORMAT)
                        .map_err(Error::ParseTimestamp)
                    {
                        Ok(time) => time,
                        Err(e) => return Some(Err(e)),
                    };
                    let parsed_date = match Date::parse(date, CPU_MONITORING_DATE_FORMAT)
                        .map_err(Error::ParseTimestamp)
                    {
                        Ok(date) => date,
                        Err(e) => return Some(Err(e)),
                    };

                    let millis = PlainDateTime::new(parsed_date, parsed_time)
                        .assume_utc()
                        .unix_timestamp_nanos()
                        / 1_000_000;
                    utc = u64::try_from(millis).ok();
                    self.1 = ParserState::EntryID;
                    break;
                } else {
                    continue;
                }
            }
        }

        if !matches!(self.1, ParserState::EntryID) {
            return None;
        }

        let header = iter.get_line()?;
        let mut tok = Tokenizer::new(header);

        if let Err(e) = tok.expect("Thread Info:").map_err(Error::InvalidFormat) {
            return Some(Err(e));
        }

        if let Err(e) = tok.expect("Thread Id :").map_err(Error::InvalidFormat) {
            return Some(Err(e));
        }

        let id = match tok.take_until(",") {
            Some(id) => id.trim().parse::<u64>().map_err(Error::ParseThreadID),
            None => {
                return Some(Err(Error::InvalidFormat(
                    tokenizer::error::Error::DelimiterNotFound(String::from(",")),
                )));
            }
        };

        let tid = match id {
            Ok(id) => id,
            Err(e) => return Some(Err(e)),
        };

        tok.skip_whitespace();
        if let Err(e) = tok.expect("has CPU usage :").map_err(Error::InvalidFormat) {
            return Some(Err(e));
        };

        let cpu = match tok.remaining() {
            "" => {
                return Some(Err(Error::InvalidFormat(
                    tokenizer::error::Error::DelimiterNotFound(String::from("<newline>")),
                )));
            }
            cpu => cpu.trim().parse::<f32>().map_err(Error::ParseCPU),
        };

        let cpu = match cpu {
            Ok(x) => x,
            Err(e) => return Some(Err(e)),
        };

        self.1 = ParserState::EntryState;

        let line = iter.get_line()?;
        let mut tok = Tokenizer::new(line);
        if let Err(e) = tok.expect("Thread Name:").map_err(Error::InvalidFormat) {
            return Some(Err(e));
        }

        tok.skip_whitespace();
        let name = tok.take_until(",");
        tok.skip_whitespace();
        if let Err(e) = tok.expect("Thread State:").map_err(Error::InvalidFormat) {
            return Some(Err(e));
        }

        let state = tok.remaining();
        let state = State::from(state.trim());

        self.1 = ParserState::EntryTrace;

        let mut frames = Vec::new();
        while let Some(line) = iter.peek_line()
            && !line.trim().is_empty()
            && line.trim_start().starts_with("at")
        {
            let frame = Frame::parse(line);
            if let Err(e) = frame {
                return Some(Err(e));
            }
            frames.push(frame.unwrap());
            iter.get_line();
        }

        let trace = if frames.is_empty() {
            None
        } else {
            Some(CPUTrace(frames))
        };

        self.0 = iter.remaining();
        self.1 = ParserState::EntryHeader;
        Some(Ok(CPUMonitoring {
            timestamp: utc.unwrap_or(0),
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
        Self(data, ParserState::EntryHeader)
    }
}

impl<'a> TryFrom<&'a [u8]> for CPUMonitoringParser<'a> {
    type Error = std::str::Utf8Error;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let value = std::str::from_utf8(value)?;
        return Ok(Self::new(value));
    }
}

#[derive(Debug)]
enum ParserState {
    EntryHeader,
    EntryID,
    EntryState,
    EntryTrace,
}

#[derive(Debug)]
pub struct CPUMonitoring<'a> {
    pub timestamp: u64,
    pub tid: u64,
    pub usage: f32,
    pub name: Option<&'a str>,
    pub state: State,
    pub trace: Option<CPUTrace<'a>>,
}

#[derive(Debug)]
pub struct CPUTrace<'a>(pub Vec<Frame<'a>>);

impl<'a> Parser<'a> for CPUTrace<'a> {
    type Error = Error;

    fn parse(data: &'a str) -> Result<Self, Self::Error> where Self: Sized {
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
    pub method: &'a str,
    pub source: &'a str,
}

impl<'a> Parser<'a> for Frame<'a> {
    type Error = Error;

    /// Expect
    /// <whitespace>at<whitespace><method>(<source>)\n
    fn parse(data: &'a str) -> Result<Self, Self::Error> {
        let mut tok = Tokenizer::new(data);
        tok.skip_whitespace();
        tok.expect("at").map_err(Error::InvalidFormat)?;
        tok.skip_whitespace();
        let method = tok
            .take_until_exclusive("(")
            .map_err(Error::InvalidFormat)?;
        let source = tok.take_within("(", ")").map_err(Error::InvalidFormat)?;
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
    use crate::parser::tokenizer;
    #[non_exhaustive]
    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Invalid Format for stack trace/cpu monitoring entry: {0:?}")]
        InvalidFormat(tokenizer::error::Error),
        #[error("Invalid Format for Timestamp: {0:?}")]
        ParseTimestamp(#[from] time::error::Parse),
        #[error("Unable to parse Thread ID: {0:?}")]
        ParseThreadID(std::num::ParseIntError),
        #[error("Unable to parse Thread CPU Usage: {0:?}")]
        ParseCPU(std::num::ParseFloatError),
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
        assert_eq!(entry.name, Some("Asset_8:XML"));
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
