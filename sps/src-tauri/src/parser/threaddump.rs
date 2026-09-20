use std::{borrow::Cow, str::Utf8Error};

use error::Error;

use crate::{
    parser::tokenizer::{self, Tokenizer},
    util,
};

use time::{format_description::BorrowedFormatItem, macros::format_description};

const THREADDUMP_TIMESTAMP_FORMAT: &[BorrowedFormatItem] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]");

pub struct ThreadDumpParser<'a>(&'a str, ParserState);
pub enum ParserState {
    Initial,
    Timestamp,
    Thread,
    FinishDump,
}

pub struct ThreadDump<'a> {
    pub threads: Vec<Thread<'a>>,
    pub timestamp: u64,
}

pub struct Thread<'a> {
    pub tid: u64,
    pub name: Cow<'a, str>,
    pub state: State<'a>,
    pub trace: Option<Trace<'a>>,
}

pub enum State<'a> {
    New,
    Runnable,
    TimedWaiting(Object<'a>),
    Waiting(Object<'a>),
    // Object, LockName, Lock Holding Thread ID, Lock Holding Thread Name
    Blocked(Object<'a>, Lock<'a>, u64, LockOwner<'a>),
    Terminated,
}

pub struct Trace<'a>(pub Vec<Element<'a>>);
pub enum Element<'a> {
    Lock(Object<'a>),
    Frame(Frame<'a>),
}

pub struct Object<'a>(pub Cow<'a, str>);
pub struct Lock<'a>(pub Cow<'a, str>);
pub struct LockOwner<'a>(pub Cow<'a, str>);
pub struct Frame<'a>(pub Cow<'a, str>, pub Cow<'a, str>);

impl<'a> ThreadDumpParser<'a> {
    pub fn new(data: &'a str) -> Self {
        Self(data, ParserState::Initial)
    }

    pub fn parse_thread(tok: &mut Tokenizer) -> Result<Thread<'a>, Error> {
        let header = tok.get_line().ok_or_else(|| {
            Error::InvalidFormat(tokenizer::error::Error::DelimiterNotFound("\n".to_owned()))
        })?;

        let mut htok = Tokenizer::new(header);
        todo!()
    }
}

impl<'a> TryFrom<&'a [u8]> for ThreadDumpParser<'a> {
    type Error = Utf8Error;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let data = std::str::from_utf8(value)?;
        Ok(Self::new(data))
    }
}

impl<'a> Iterator for ThreadDumpParser<'a> {
    type Item = Result<ThreadDump<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut tok = Tokenizer::new(self.0);
        tok.skip_whitespace();
        if tok.is_empty() {
            return None;
        }

        while let Some(line) = tok.peek_line() {
            if line.trim_start().starts_with("Thread dump") {
                self.1 = ParserState::Timestamp;
                break;
            }
            tok.get_line()?;
        }

        match tok.expect("Thread dump") {
            Ok(()) => (),
            Err(e) => return Some(Err(Error::from(e))),
        };

        tok.skip_whitespace();
        match tok.expect(":") {
            Err(e) => {
                self.0 = tok.remaining();
                return Some(Err(Error::from(e)));
            }
            _ => (),
        };

        match tok.take_until(":") {
            Some(_) => (),
            None => {
                self.0 = tok.remaining();
                return Some(Err(Error::from(
                    tokenizer::error::Error::DelimiterNotFound(":".to_owned()),
                )));
            }
        };

        let timestamp = tok.get_line()?;
        let timestamp =
            match util::utc_unix_timestamp_millis(timestamp, THREADDUMP_TIMESTAMP_FORMAT) {
                Ok(timestamp) => timestamp,
                Err(e) => {
                    self.0 = tok.remaining();
                    return Some(Err(Error::from(e)));
                }
            };

        self.1 = ParserState::Thread;
        let mut threads = Vec::new();
        while let Some(line) = tok.peek_line() {
            if line.trim_start().starts_with("TriggeredTime") {
                self.1 = ParserState::FinishDump;
                break;
            }

            if line.trim_start().starts_with("\"") {
                self.1 = ParserState::Thread;
                let thread = match Self::parse_thread(&mut tok) {
                    Ok(thread) => thread,
                    Err(e) => Some(Err(Error::from(e))),
                };
            }
        }
        todo!()
    }
}

pub mod error {
    use crate::parser::tokenizer;
    use crate::types::TimestampError;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Invalid format in parsing ThreadDump")]
        InvalidFormat(#[from] tokenizer::error::Error),

        #[error("Invalid Timestamp in ThreadDump meta info")]
        Timestamp(#[from] TimestampError),
    }
}
