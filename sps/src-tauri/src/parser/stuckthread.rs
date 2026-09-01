use error::Error;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use time::{format_description::BorrowedFormatItem, macros::format_description};

use crate::{
    parser::tokenizer::{self, Parser, Tokenizer},
    util,
};

const STUCKTHREAD_TIME_FORMAT: &[BorrowedFormatItem] =
    format_description!("[hour]:[minute]:[second].[subsecond]");
const STUCKTHREAD_DATE_FORMAT: &[BorrowedFormatItem] = format_description!("[day]-[month]-[year]");

pub struct StuckthreadParser<'a>(&'a str, ParserState);

enum ParserState {
    Initial,
    Header,
}

impl<'a> Iterator for StuckthreadParser<'a> {
    type Item = Result<Stuckthread<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut tok = Tokenizer::new(self.0);
        tok.skip_whitespace();
        while let Some(line) = tok.peek_line()
            && line.trim_start().starts_with("[")
        {
            self.1 = ParserState::Header;
            break;
        }

        if !matches!(self.1, ParserState::Header) {
            self.0 = tok.remaining();
            return Some(Err(Error::HeaderNotFound));
        }

        let header = tok.get_line()?;
        let has_stacktrace = tok.peek_line()?.trim().starts_with(Self::PREAMBLE);
        let mut thread = match Self::parse_header(header, has_stacktrace) {
            Ok(t) => t,
            Err(e) => return Some(Err(e)),
        };

        if let Stuckthread::Begin { ref mut trace, .. } = thread
            && has_stacktrace
        {
            let _ = tok.get_line()?;
            while let Some(line) = tok.peek_line()
                && (!line.trim().starts_with("[") || line.trim().starts_with("at"))
            {
                let frame = match Frame::parse(line) {
                    Ok(f) => f,
                    Err(e) => return Some(Err(e)),
                };
                trace.0.push(frame);
            }
        }
        Some(Ok(thread))
    }
}

impl<'a> StuckthreadParser<'a> {
    const PREAMBLE: &'static str = "java.lang.Throwable";
    pub fn new(data: &'a str) -> Self {
        Self(data, ParserState::Initial)
    }

    pub fn parse_header<'b>(data: &'b str, has_stacktrace: bool) -> Result<Stuckthread<'b>, Error> {
        let mut tok = Tokenizer::new(data);
        let raw_time = tok.take_within("[", "]")?;
        let raw_date = tok.take_within("[", "]")?;
        let start = util::unix_timestamp_millis(
            raw_time,
            raw_date,
            STUCKTHREAD_TIME_FORMAT,
            STUCKTHREAD_DATE_FORMAT,
        )?;

        let _ = tok.take_until("::").ok_or_else(|| {
            Error::InvalidFormat(tokenizer::error::Error::DelimiterNotFound("::".to_string()))
        })?;

        let name = tok.take_within("[", "]")?.trim().into();
        let tid = tok.take_within("[", "]")?.trim().parse()?;
        let duration = tok.take_within("[", "]")?.trim();
        let duration = util::parse_comma_separated_u64(duration)?;

        if !has_stacktrace {
            let active = match tok.take_within("[", "]") {
                Ok(active) => Some(active.trim().parse()?),
                Err(e) => match e {
                    tokenizer::error::Error::DelimiterNotFound(ref x) => {
                        if x != "[" {
                            return Err(Error::InvalidFormat(e));
                        } else {
                            None
                        }
                    }
                    _ => return Err(Error::InvalidFormat(e)),
                },
            };

            Ok(Stuckthread::End {
                tid,
                name,
                duration,
                active,
            })
        } else {
            // This information is not needed: since [2/11/26 3:26 PM], but is still important for
            // parsing
            let _ = tok.take_within("[", "]")?;
            let request = tok.take_within("[", "]")?.trim().into();
            // This information is not needed: configured threshold for this StuckThreadDetectionValve is [10] seconds
            // but is still important for parsing
            let _ = tok.take_within("[", "]")?;
            let active = match tok.take_within("[", "]") {
                Ok(active) => Some(active),
                Err(e) => match e {
                    tokenizer::error::Error::DelimiterNotFound(ref x) => {
                        if x != "[" {
                            return Err(Error::InvalidFormat(e));
                        } else {
                            None
                        }
                    }
                    _ => return Err(Error::InvalidFormat(e)),
                },
            }
            .and_then(|n| n.parse().ok());
            Ok(Stuckthread::Begin {
                start,
                tid,
                name,
                request,
                trace: Default::default(),
                active,
            })
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Stuckthread<'a> {
    Begin {
        start: u64,
        tid: u64,
        name: Cow<'a, str>,
        request: Cow<'a, str>,
        trace: Trace<'a>,
        active: Option<u64>,
    },
    End {
        tid: u64,
        name: Cow<'a, str>,
        duration: u64,
        active: Option<u64>,
    },
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Trace<'a>(pub Vec<Frame<'a>>);

#[derive(Debug, Serialize, Deserialize)]
pub struct Frame<'a> {
    pub source: Cow<'a, str>,
    pub method: Cow<'a, str>,
}

impl<'a> Parser<'a> for Frame<'a> {
    type Error = Error;

    fn parse(data: &'a str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut tok = Tokenizer::new(data);
        tok.skip_whitespace();
        tok.expect("at")?;
        tok.skip_whitespace();
        let method = tok.take_until_exclusive("(")?.into();
        let source = tok.take_within("(", ")")?.into();
        Ok(Frame { source, method })
    }
}

pub mod error {
    use crate::{
        parser::tokenizer,
        types::{ParseInt, TimestampError},
    };

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Invalid Format for stuckthread/traces: {0}")]
        InvalidFormat(#[from] tokenizer::error::Error),
        #[error("Time Stamp parse error: {0}")]
        ParseTime(#[from] TimestampError),
        #[error("Parsing duration: {0}")]
        ParseDuration(#[from] ParseInt),
        #[error("Cannot find header in stuckthread entry")]
        HeaderNotFound,
        #[error("Cannot find header in stuckthread entry")]
        ParseTID(#[from] std::num::ParseIntError),
    }
}

#[cfg(test)]
pub mod test {
    use crate::parser::{stuckthread::Frame, tokenizer::Parser};

    #[test]
    fn stuckthread_frame_parse() {
        let data = "	at java.base@17.0.17/java.lang.Thread.run(Unknown Source)";
        let frame = Frame::parse(data);
        assert!(frame.is_ok(), "Error during parsing: {}", frame.unwrap_err());
        let frame = frame.unwrap();
        assert_eq!(frame.method, "java.base@17.0.17/java.lang.Thread.run"); 
        assert_eq!(frame.source, "Unknown Source");
    }
}
