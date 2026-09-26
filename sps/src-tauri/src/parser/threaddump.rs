use std::{borrow::Cow, ops::Deref, str::Utf8Error};

use error::Error;
use serde::{Deserialize, Serialize};

use crate::{parser::tokenizer::Tokenizer, util};

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

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum State<'a> {
    New,
    Runnable,
    Terminated,
    TimedWaiting {
        waiting_on: Option<Object<'a>>,
    },
    Waiting {
        waiting_on: Object<'a>,
        lock: Option<Lock<'a>>,
        lock_owner_tid: Option<u64>,
        lock_owner_name: Option<LockOwner<'a>>,
    },
    Blocked {
        blocked_on: Object<'a>,
        lock: Lock<'a>,
        lock_owner_tid: u64,
        lock_owner_name: LockOwner<'a>,
    },
}

impl<'a> State<'a> {
    pub fn as_str(&self) -> &'static str {
        match self {
            State::New => "NEW",
            State::Runnable => "RUNNABLE",
            State::Terminated => "TERMINATED",
            State::TimedWaiting { .. } => "TIMED_WAITING",
            State::Waiting { .. } => "WAITING",
            State::Blocked { .. } => "BLOCKED",
        }
    }
}

#[derive(Debug)]
pub struct Trace<'a>(pub Vec<Element<'a>>);
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Element<'a> {
    Lock {
        object: Object<'a>,
    },
    Frame {
        method: Cow<'a, str>,
        source: Cow<'a, str>,
    },
}

impl<'a> AsRef<Vec<Element<'a>>> for Trace<'a> {
    fn as_ref(&self) -> &Vec<Element<'a>> {
        &self.0
    }
}

impl<'a> Deref for Trace<'a> {
    type Target = Vec<Element<'a>>;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Object<'a>(pub Cow<'a, str>);
#[derive(Debug, Deserialize, Serialize)]
pub struct Lock<'a>(pub Cow<'a, str>);
#[derive(Debug, Deserialize, Serialize)]
pub struct LockOwner<'a>(pub Cow<'a, str>);

impl<'a> ThreadDumpParser<'a> {
    pub fn new(data: &'a str) -> Self {
        Self(data, ParserState::Initial)
    }

    pub fn parse_thread(tok: &mut Tokenizer<'a>) -> Result<Thread<'a>, Error> {
        let name = tok.take_within("\"", "\"")?.into();
        tok.skip_whitespace();
        tok.expect("Id=")?;
        let tid = tok.take_until_fallible(" ")?.parse()?;
        let state = Self::parse_thread_state(&mut *tok)?;

        tok.skip_whitespace();
        if tok.peek("\"") {
            return Ok(Thread {
                tid,
                name,
                state,
                trace: None,
            });
        }

        let mut traces = Vec::new();
        while let Some(line) = tok.peek_line() {
            let line = line.trim_start();
            let mut ttok = Tokenizer::new(line);
            ttok.skip_whitespace();
            if ttok.peek("- locked") {
                ttok.expect("- locked")?;
                let object = ttok.remaining().trim().into();
                traces.push(Element::Lock {
                    object: Object(object),
                });
            } else if line.contains("(") {
                let method = ttok.take_until_exclusive("(")?.trim().into();
                let source = ttok.take_within("(", ")")?.into();
                traces.push(Element::Frame { method, source });
            } else {
                let _ = tok.get_line();
                break;
            }
            let _ = tok.get_line();
        }

        let trace = if traces.is_empty() {
            None
        } else {
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
        if tok.peek("RUNNABLE") {
            tok.expect("RUNNABLE")?;
            tok.skip_whitespace();
            Ok(State::Runnable)
        } else if tok.peek("TERMINATED") {
            tok.expect("TERMINATED")?;
            tok.skip_whitespace();
            Ok(State::Terminated)
        } else if tok.peek("NEW") {
            tok.expect("NEW")?;
            tok.skip_whitespace();
            Ok(State::New)
        } else if tok.peek("TIMED_WAITING") {
            tok.expect("TIMED_WAITING")?;
            tok.skip_whitespace();
            if tok.peek("on") {
                tok.expect("on")?;
                tok.skip_whitespace();
                let object = tok.take_until_fallible("\n")?.into();
                Ok(State::TimedWaiting {
                    waiting_on: Some(Object(object)),
                })
            } else {
                Ok(State::TimedWaiting { waiting_on: None })
            }
        } else if tok.peek("WAITING") {
            tok.expect("WAITING")?;
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
                Ok(State::Waiting {
                    waiting_on: Object(object),
                    lock: Some(Lock(lockname)),
                    lock_owner_tid: Some(owner_id),
                    lock_owner_name: Some(LockOwner(owner_name)),
                })
            } else {
                Ok(State::Waiting {
                    waiting_on: Object(object),
                    lock: None,
                    lock_owner_tid: None,
                    lock_owner_name: None,
                })
            }
        } else if tok.peek("BLOCKED") {
            tok.expect("BLOCKED")?;
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
            Ok(State::Blocked {
                blocked_on: Object(object),
                lock: Lock(lock),
                lock_owner_tid: owner,
                lock_owner_name: LockOwner(owner_name),
            })
        } else {
            Err(Error::InvalidState)
        }
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

        self.1 = ParserState::Initial;
        while let Some(line) = tok.peek_line() {
            if line.trim_start().starts_with("Thread dump") {
                self.1 = ParserState::Timestamp;
                break;
            }
            tok.get_line()?;
        }

        if let ParserState::Initial = self.1 {
            return None;
        }

        match tok.expect("Thread dump") {
            Ok(()) => (),
            Err(e) => return Some(Err(Error::from(e))),
        };

        tok.skip_whitespace();

        if let Err(e) = tok.expect(":") {
            self.0 = tok.remaining();
            return Some(Err(Error::from(e)));
        }

        if let Err(e) = tok.take_until_fallible(":") {
            self.0 = tok.remaining();
            return Some(Err(Error::from(e)));
        }

        tok.skip_whitespace();
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
            if line.trim_start().starts_with("TriggeredTime")
                || line.trim_start().starts_with("Thread dump")
            {
                self.1 = ParserState::FinishDump;
                break;
            }

            if line.trim_start().starts_with("\"") {
                self.1 = ParserState::Thread;
                let thread = match Self::parse_thread(&mut tok) {
                    Ok(thread) => thread,
                    Err(e) => {
                        self.0 = tok.remaining();
                        return Some(Err(e));
                    }
                };

                threads.push(thread);
            } else {
                tok.get_line()?;
            }
        }
        self.0 = tok.remaining();
        Some(Ok(ThreadDump { threads, timestamp }))
    }
}

#[cfg(test)]
pub mod test {
    use std::{assert_matches, ops::Deref};

    use crate::{
        parser::{
            threaddump::{Object, State, ThreadDumpParser},
            tokenizer::Tokenizer,
        },
        util,
    };

    #[test]
    fn threaddump_thread_header_only() {
        let line = r#""Reference Handler"  Id=2  Java.lang.Thread.State: RUNNABLE"#;
        let mut tok = Tokenizer::new(line);
        let result = ThreadDumpParser::parse_thread(&mut tok);
        assert!(
            result.is_ok(),
            "Error during parsing: {}",
            result.unwrap_err()
        );
        let thread = result.unwrap();
        assert_matches!(thread.state, State::Runnable);
        assert_eq!(thread.tid, 2);
        assert_eq!(thread.name, "Reference Handler");
        assert_matches!(thread.trace, None);

        let line = r#""Signal Dispatcher"  Id=4  Java.lang.Thread.State: RUNNABLE"#;
        let mut tok = Tokenizer::new(line);

        let result = ThreadDumpParser::parse_thread(&mut tok);
        assert!(
            result.is_ok(),
            "Error during parsing: {}",
            result.unwrap_err()
        );
        let thread = result.unwrap();
        assert_matches!(thread.state, State::Runnable);
        assert_eq!(thread.tid, 4);
        assert_eq!(thread.name, "Signal Dispatcher");
        assert_matches!(thread.trace, None);
    }

    #[test]
    fn threaddump_with_traces() {
        let map = util::map_file("test/threaddump/threaddump_with_traces.txt").unwrap();
        let data = std::str::from_utf8(map.deref()).unwrap();
        let mut tok = Tokenizer::new(data);
        let result = ThreadDumpParser::parse_thread(&mut tok);
        assert!(
            result.is_ok(),
            "Error during parsing: {}",
            result.unwrap_err()
        );
        let thread = result.unwrap();
        assert_matches!(
            thread.state,
            State::Waiting {
                waiting_on: Object(_),
                lock: None,
                lock_owner_tid: None,
                lock_owner_name: None
            }
        );
        assert_eq!(thread.tid, 3);
        assert_eq!(thread.name, "Finalizer");
        assert_matches!(thread.trace, Some(_));
        if let Some(trace) = thread.trace {
            assert_eq!(trace.len(), 4);
        }
    }

    #[test]
    fn threaddump_full_dump() {
        let map = util::map_file("test/threaddump/threaddump_full.txt").unwrap();
        let mut parser = ThreadDumpParser::try_from(map.deref()).unwrap();
        let result = parser.next();
        assert!(result.is_some(), "No results from parser");
        let result = result.unwrap();
        assert!(
            result.is_ok(),
            "Error during parsing: {}",
            result.unwrap_err()
        );
        let result = result.unwrap();
        assert_eq!(result.threads.len(), 293);
        dbg!(result);
    }

    #[test]
    fn threaddump_full() {
        let mut thread_count = 0;
        let map = util::map_file("test/threaddump/threaddump0.txt").unwrap();
        let parser = ThreadDumpParser::try_from(map.deref()).unwrap();
        let mut count = 0;
        for dump in parser {
            assert!(
                dump.is_ok(),
                "Error during parsing dump: {}",
                dump.unwrap_err()
            );

            thread_count += dump.unwrap().threads.len();
            count += 1;
        }

        assert_eq!(count, 6);

        let map = util::map_file("test/threaddump/threaddump1.txt").unwrap();
        let parser = ThreadDumpParser::try_from(map.deref()).unwrap();
        let mut count = 0;
        for dump in parser {
            assert!(
                dump.is_ok(),
                "Error during parsing dump: {}",
                dump.unwrap_err()
            );

            thread_count += dump.unwrap().threads.len();
            count += 1;
        }

        assert_eq!(count, 9);

        let map = util::map_file("test/threaddump/threaddump2.txt").unwrap();
        let parser = ThreadDumpParser::try_from(map.deref()).unwrap();
        let mut count = 0;
        for dump in parser {
            assert!(
                dump.is_ok(),
                "Error during parsing dump: {}",
                dump.unwrap_err()
            );

            thread_count += dump.unwrap().threads.len();
            count += 1;
        }

        assert_eq!(count, 9);

        let map = util::map_file("test/threaddump/threaddump3.txt").unwrap();
        let parser = ThreadDumpParser::try_from(map.deref()).unwrap();
        let mut count = 0;
        for dump in parser {
            assert!(
                dump.is_ok(),
                "Error during parsing dump: {}",
                dump.unwrap_err()
            );

            thread_count += dump.unwrap().threads.len();
            count += 1;
        }

        assert_eq!(count, 9);

        let map = util::map_file("test/threaddump/threaddump4.txt").unwrap();
        let parser = ThreadDumpParser::try_from(map.deref()).unwrap();
        let mut count = 0;
        for dump in parser {
            assert!(
                dump.is_ok(),
                "Error during parsing dump: {}",
                dump.unwrap_err()
            );

            thread_count += dump.unwrap().threads.len();
            count += 1;
        }

        assert_eq!(count, 6);
        assert_eq!(thread_count, 12065);
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

        #[error("Invalid Thread State found")]
        InvalidState,
    }
}
