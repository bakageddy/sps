use std::{borrow::Cow, str::Utf8Error};

use error::Error;

use crate::{
    parser::tokenizer::{self, Tokenizer},
    util,
};

use time::{format_description::BorrowedFormatItem, macros::format_description};

const THREADDUMP_TIMESTAMP_FORMAT: &[BorrowedFormatItem] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]");

#[derive(Debug)]
pub struct ThreadDumpParser<'a>(&'a str, ParserState);
#[derive(Debug)]
pub enum ParserState {
    Initial,
    Timestamp,
    Thread,
    FinishDump,
}

#[derive(Debug)]
pub struct ThreadDump<'a> {
    pub threads: Vec<Thread<'a>>,
    pub timestamp: u64,
}

#[derive(Debug)]
pub struct Thread<'a> {
    pub tid: u64,
    pub name: Cow<'a, str>,
    pub state: State<'a>,
    pub trace: Option<Trace<'a>>,
}

#[derive(Debug)]
pub enum State<'a> {
    New,
    Runnable,
    TimedWaiting(Option<Object<'a>>),
    Waiting(Object<'a>, Option<Lock<'a>>, Option<u64>, Option<LockOwner<'a>>),
    // Object, LockName, Lock Holding Thread ID, Lock Holding Thread Name
    Blocked(Object<'a>, Lock<'a>, u64, LockOwner<'a>),
    Terminated,
}

#[derive(Debug)]
pub struct Trace<'a>(pub Vec<Element<'a>>);
#[derive(Debug)]
pub enum Element<'a> {
    Lock(Object<'a>),
    Frame(Frame<'a>),
}

#[derive(Debug)]
pub struct Object<'a>(pub Cow<'a, str>);
#[derive(Debug)]
pub struct Lock<'a>(pub Cow<'a, str>);
#[derive(Debug)]
pub struct LockOwner<'a>(pub Cow<'a, str>);
#[derive(Debug)]
pub struct Frame<'a>(pub Cow<'a, str>, pub Cow<'a, str>);

impl<'a> ThreadDumpParser<'a> {
    pub fn new(data: &'a str) -> Self {
        Self(data, ParserState::Initial)
    }

    pub fn parse_thread(tok: &mut Tokenizer<'a>) -> Result<Thread<'a>, Error> {
        let name = tok.take_within("\"", "\"")?.into();
        tok.skip_whitespace();
        tok.expect("Id=")?;
        let id = tok.take_until_fallible(" ")?;
        let tid = id.parse()?;
        let state = Self::parse_thread_state(&mut *tok)?;
        tok.skip_whitespace();
        let trace = if tok.peek("\"") {
            None
        } else {
            let mut traces = Vec::new();
            while let Some(line) = tok.peek_line() {
                if line.trim_start().starts_with("\"") || line.trim_start().is_empty() {
                    break;
                }
                let mut ttok = Tokenizer::new(line);
                ttok.skip_whitespace();
                if ttok.peek("- locked") {
                    ttok.expect("- locked")?;
                    let object = ttok.take_until_fallible("\n")?.into();
                    traces.push(Element::Lock(Object(object)));
                } else {
                    let method = ttok.take_until_exclusive("(")?.into();
                    let source = ttok.take_within("(", ")")?.into();
                    let frame = Frame(method, source);
                    traces.push(Element::Frame(frame));
                }
                tok.get_line();
            }
            Some(Trace(traces))
        };

        Ok(Thread {
            tid,
            name,
            state,
            trace,
        })
    }

    pub fn parse_thread_state(tok: &mut Tokenizer<'a>) -> Result<State<'a>, Error> {
        tok.skip_whitespace();
        tok.expect("Java.lang.Thread.State:")?;
        tok.skip_whitespace();
        let state = tok.take_until_fallible(" ")?;
        let state = match state {
            "RUNNABLE" => {
                State::Runnable
            },
            "TERMINATED" => {
                State::Terminated
            },
            "NEW" => {
                State::New
            },
            "TIMED_WAITING" => {
                tok.skip_whitespace();
                if tok.peek("on") {
                    tok.expect("on")?;
                    tok.skip_whitespace();
                    let object = tok.take_until_fallible("\n")?.into();
                    State::TimedWaiting(Some(Object(object)))
                } else {
                    State::TimedWaiting(None)
                }
            },
            "WAITING" => {
                tok.skip_whitespace();
                tok.expect("on")?;
                tok.skip_whitespace();
                let object = tok.take_until_fallible("\n")?.into();
                tok.skip_whitespace();
                if tok.peek("LockName:") {
                    tok.expect("LockName:")?;
                    tok.skip_whitespace();
                    let lockname = tok.take_until_fallible(" ")?.into();
                    tok.skip_whitespace();
                    tok.expect("Owner Id:")?;
                    tok.skip_whitespace();
                    let owner_id = tok.take_until_fallible(" ")?.parse()?;
                    tok.skip_whitespace();
                    tok.expect("Owner Name:")?;
                    tok.skip_whitespace();
                    let owner_name = tok.take_until_fallible("\n")?.into();
                    State::Waiting(Object(object), Some(Lock(lockname)), Some(owner_id), Some(LockOwner(owner_name)))
                } else {
                    State::Waiting(Object(object), None, None, None)
                }
            },
            "BLOCKED" => {
                tok.skip_whitespace();
                tok.expect("waiting to lock")?;
                tok.skip_whitespace();
                let object = tok.take_until_fallible("\n")?.into();
                tok.skip_whitespace();
                tok.expect("LockName:")?;
                tok.skip_whitespace();
                let lock = tok.take_until_fallible(" ")?.into();
                tok.skip_whitespace();
                tok.expect("Owner Id:")?;
                tok.skip_whitespace();
                let owner = tok.take_until_fallible(" ")?.parse()?;
                tok.skip_whitespace();
                tok.expect("Owner Name:")?;
                tok.skip_whitespace();
                let owner_name = tok.take_until_fallible("\n")?.into();
                tok.skip_whitespace();
                State::Blocked(Object(object), Lock(lock), owner, LockOwner(owner_name))
            },
            _ => {
                return Err(Error::InvalidState(state.to_owned()));
            }
        };

        Ok(state)
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
                    Err(e) => return Some(Err(Error::from(e))),
                };

                threads.push(thread);
            }
        }
        Some(Ok(ThreadDump {
            threads,
            timestamp
        }))
    }
}

#[cfg(test)]
pub mod test {
    use std::assert_matches;

use crate::parser::{threaddump::{State, ThreadDumpParser}, tokenizer::Tokenizer};

    #[test]
    fn threaddump_thread_header_only() {
        let line = r#""Reference Handler"  Id=2  Java.lang.Thread.State: RUNNABLE\n"#;
        let mut tok = Tokenizer::new(line);
        let result = ThreadDumpParser::parse_thread(&mut tok);
        assert!(result.is_ok(), "Error during parsing: {}", result.unwrap_err());
        let thread = result.unwrap(); 
        assert_matches!(thread.state, State::Runnable);
        assert_eq!(thread.tid, 2);
        assert_eq!(thread.name, "Reference Handler");
    }
}

pub mod error {
    use std::num::ParseIntError;

    use crate::parser::tokenizer;
    use crate::types::TimestampError;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Invalid format in parsing ThreadDump: {0}")]
        InvalidFormat(#[from] tokenizer::error::Error),

        #[error("Invalid Timestamp in ThreadDump meta info: {0}")]
        Timestamp(#[from] TimestampError),

        #[error("Parsing Integer: {0}")]
        IntegerParse(#[from] ParseIntError),

        #[error("Invalid Thread State: {0}")]
        InvalidState(String),
    }
}
