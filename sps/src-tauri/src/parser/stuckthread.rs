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

        let time = match tok.take_within("[", "]").map_err(Error::InvalidFormat) {
            Ok(time) => time,
            Err(e) => return Some(Err(e)),
        };
        let date = match tok.take_within("[", "]").map_err(Error::InvalidFormat) {
            Ok(time) => time,
            Err(e) => return Some(Err(e)),
        };

        let timestamp = util::unix_timestamp_millis(
            time,
            date,
            STUCKTHREAD_TIME_FORMAT,
            STUCKTHREAD_DATE_FORMAT,
        )
        .unwrap_or(0);

        let _ = match tok.take_until("::").ok_or_else(|| {
            Error::InvalidFormat(tokenizer::error::Error::DelimiterNotFound("::".to_string()))
        }) {
            Ok(_) => {}
            Err(e) => return Some(Err(e)),
        };

        let name = match tok.take_within("[", "]").map_err(Error::InvalidFormat) {
            Ok(x) => x.trim(),
            Err(e) => return Some(Err(e))
        };

        let id = match tok.take_within("[", "]").map_err(Error::InvalidFormat) {
            Ok(x) => x,
            Err(e) => return Some(Err(e))
        };

        let tid = match id.trim().parse().map_err(Error::ParseTID) {
            Ok(x) => x,
            Err(e) => return Some(Err(e))
        };

        let duration = match tok.take_within("[", "]").map_err(Error::InvalidFormat) {
            Ok(x) => x,
            Err(e) => return Some(Err(e))
        };

        let sentinel = match tok.take_within("[", "]") {
            Ok(x) => x,
            Err(e) => {
                tokenizer::error::Error::NotEnoughData => todo!(),
                tokenizer::error::Error::Expected { expected, got } => todo!(),
                tokenizer::error::Error::DelimiterNotFound(x) => {
                    if (x == "[") {
                        return Ok(Stuckthread::End { tid, name, duration: () })
                    }
                }
                tokenizer::error::Error::DelimiterNotFound(_) => todo!(),
            }
        };

        todo!()
    }
}

impl<'a> StuckthreadParser<'a> {
    const PREAMBLE: &'static str = "java.lang.Throwable";
    pub fn new(data: &'a str) -> Self {
        Self(data, ParserState::Initial)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Stuckthread<'a> {
    Begin {
        start: u64,
        tid: u64,
        name: Cow<'a, str>,
        active: u64,
        request: Cow<'a, str>,
        trace: Trace<'a>,
    },
    End {
        tid: u64,
        name: Cow<'a, str>,
        duration: u64,
    },
}

#[derive(Debug, Serialize, Deserialize)]
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
    use crate::parser::tokenizer;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Invalid Format for stuckthread/traces: {0}")]
        InvalidFormat(#[from] tokenizer::error::Error),
        #[error("Cannot find header in stuckthread entry")]
        HeaderNotFound,
        #[error("Cannot find header in stuckthread entry")]
        ParseTID(#[from] std::num::ParseIntError),
    }
}
