use std::collections::HashMap;

use crate::{error::threaddump::Parse, scanner::Scanner, util::ToUnixMillis};
use time::{PrimitiveDateTime, format_description::FormatItem, macros::format_description};
use tracing::warn;

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Object<'a> {
    pub class: &'a str,
    pub identity: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Source<'a> {
    NativeMethod,
    UnknownSource,
    Generated(&'a str),
    Filename { file: &'a str, line_number: i64 },
}

#[derive(Debug, PartialEq, Eq)]
pub enum Element<'a> {
    Lock(Object<'a>),
    Elem { frame: &'a str, source: Source<'a> },
}

#[derive(Debug, PartialEq, Eq)]
pub struct StackTrace<'a> {
    pub elems: Vec<Element<'a>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct LockInfo<'a> {
    pub owner_id: ThreadID,
    pub owner_name: Option<&'a str>,
    pub object: Object<'a>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ThreadState<'a> {
    New,
    Terminated,
    Runnable,
    BlockedToLock(Option<LockInfo<'a>>),
    TimedWaiting,
    TimedWaitingOn(Object<'a>),
    Waiting,
    WaitingOn(Object<'a>),
    WaitingToLock(LockInfo<'a>),
}

pub type ThreadID = i64;

#[derive(Debug, PartialEq, Eq)]
pub struct Thread<'a> {
    pub thread_id: ThreadID,
    pub state: ThreadState<'a>,
    pub stacktrace: Option<StackTrace<'a>>,
    pub thread_name: Option<&'a str>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ThreadDump<'a> {
    pub threads: HashMap<ThreadID, Thread<'a>>,
    pub triggered_unix_ms: i64,
    pub snapshot: u8,
}

pub struct ThreadDumpStreamer<'a>(pub &'a [u8]);

impl<'a> TryFrom<&'a str> for LockInfo<'a> {
    type Error = Parse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        scanner.skip_whitespace();
        scanner
            .expect("LockName: ")
            .map_err(|_| Parse::ExpectedLockName)?;
        let object = scanner.take_until(" ").ok_or(Parse::ExpectedLockObject)?;
        let object = Object::try_from(object)?;

        scanner.skip_whitespace();
        scanner
            .expect("Owner Id: ")
            .map_err(|_| Parse::ExpectedOwnerId)?;

        let owner_id: ThreadID = scanner
            .take_until(" ")
            .ok_or(Parse::ExpectedOwnerId)?
            .parse()
            .map_err(Parse::ThreadIdParse)?;

        scanner.skip_whitespace();
        scanner
            .expect("Owner Name: ")
            .map_err(|_| Parse::ExpectedOwnerName)?;
        let owner_name = scanner.remaining().trim();
        let owner_name = if owner_name.is_empty() {
            None
        } else {
            Some(owner_name)
        };

        Ok(LockInfo {
            owner_id,
            owner_name,
            object,
        })
    }
}

impl<'a> TryFrom<&'a str> for Object<'a> {
    type Error = Parse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
    }
}

impl<'a> TryFrom<&'a str> for Source<'a> {
    type Error = Parse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "Unknown Source" => return Ok(Source::UnknownSource),
            "Native Method" => return Ok(Source::NativeMethod),
            _ => {}
        };

        if value.contains("$") || !value.contains(":") {
            return Ok(Source::Generated(value));
        }

        let (file, lineno) = value.split_once(":").ok_or(Parse::ColonNotFound)?;
        Ok(Source::Filename {
            file,
            line_number: lineno.parse::<i64>()?,
        })
    }
}

impl<'a> TryFrom<&'a str> for Element<'a> {
    type Error = Parse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        scanner.skip_whitespace();
        if scanner.peek_expect("- locked") {
            scanner.expect("- locked").expect("SAFETY: CHECKED");
            scanner.skip_whitespace();
            let result = Object::try_from(scanner.remaining())?;
            return Ok(Element::Lock(result));
        }

        let frame = scanner
            .take_until_inclusive("(")
            .ok_or(Parse::OpenParenNotFound)?;
        let source = scanner
            .take_within("(", ")")
            .map_err(|_| Parse::CloseParenNotFound)?;
        let source = Source::try_from(source)?;
        Ok(Element::Elem { frame, source })
    }
}

impl<'a> TryFrom<&'a str> for StackTrace<'a> {
    type Error = Parse;
    // NOTE: This is simple enough. DO NOT REFACTOR WITH SCANNER
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let value = value.trim();
        let mut frames = Vec::new();
        for line in value.lines() {
            let result = Element::try_from(line)?;
            frames.push(result);
        }

        Ok(StackTrace { elems: frames })
    }
}

impl<'a> ThreadState<'a> {
    const PREAMBLE: &'static str = "Java.lang.Thread.State:";
}

impl<'a> TryFrom<&'a str> for ThreadState<'a> {
    type Error = Parse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        scanner.skip_whitespace();
        scanner
            .expect(ThreadState::PREAMBLE)
            .map_err(|_| Parse::ExpectedPreamble)?;
        scanner.skip_whitespace();

        if let Some(raw_state) = scanner.peek_until(" waiting to lock ") {
            let raw_state = raw_state.trim();
            if raw_state.trim() != "BLOCKED" {
                return Err(Parse::ExpectedLockObject);
            }
            Ok(ThreadState::BlockedToLock(None))
        } else if let Some(raw_state) = scanner.peek_until(" on ") {
            match raw_state.trim() {
                "WAITING" => {
                    _ = scanner.take_until("on").expect("SAFETY: CHECKED");
                    scanner.skip_whitespace();
                    let object = scanner.remaining();
                    let object = Object::try_from(object)?;
                    Ok(ThreadState::WaitingOn(object))
                }
                "TIMED_WAITING" => {
                    _ = scanner.take_until("on").expect("SAFETY: CHECKED");
                    scanner.skip_whitespace();
                    let object = scanner.remaining();
                    let object = Object::try_from(object)?;
                    Ok(ThreadState::TimedWaitingOn(object))
                }
                _ => {
                    println!("STATE: {raw_state}");
                    Err(Parse::ExpectedWaitObject)
                }
            }
        } else {
            match scanner.remaining().trim() {
                "NEW" => Ok(ThreadState::New),
                "TERMINATED" => Ok(ThreadState::Terminated),
                "RUNNABLE" => Ok(ThreadState::Runnable),
                "TIMED_WAITING" => Ok(ThreadState::TimedWaiting),
                "WAITING" => Ok(ThreadState::Waiting),
                _ => Err(Parse::UnexpectedThreadState),
            }
        }
    }
}

impl<'a> TryFrom<&'a str> for Thread<'a> {
    type Error = Parse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        scanner.skip_whitespace();
        let thread_name = scanner
            .take_within("\"", "\"")
            .map_err(|_| Parse::ThreadNameExtraction)?
            .trim();
        let thread_name = if thread_name.is_empty() {
            None
        } else {
            Some(thread_name)
        };

        scanner.skip_whitespace();
        scanner
            .expect("Id=")
            .map_err(|_| Parse::ThreadIdExtraction)?;

        let id = scanner
            .take_until_inclusive(" ")
            .ok_or(Parse::ThreadIdExtraction)?;
        let thread_id: i64 = id.parse().map_err(Parse::ThreadIdParse)?;
        scanner.skip_whitespace();

        if scanner.peek_until("\n").is_none() {
            let header = scanner.remaining();
            let state = ThreadState::try_from(header)?;
            return Ok(Thread {
                thread_id,
                state,
                thread_name,
                stacktrace: None
            })
        }

        let header = scanner.take_until("\n").ok_or(Parse::ThreadHeaderExtraction)?;
        let state = ThreadState::try_from(header)?;
        scanner.skip_whitespace();
        let state = if scanner.peek_expect("LockName: ") {
            let lock_info = scanner
                .take_until("\n")
                .ok_or(Parse::ExpectedValidLockInformation)?;
            let lock_info = LockInfo::try_from(lock_info)?;
            match state {
                ThreadState::WaitingOn(_) => ThreadState::WaitingToLock(lock_info),
                ThreadState::BlockedToLock(None) => ThreadState::BlockedToLock(Some(lock_info)),
                _ => return Err(Parse::UnexpectedThreadState),
            }
        } else {
            state
        };

        let data = scanner.remaining().trim();
        let mut stacktrace: Option<StackTrace<'a>> = None;
        if !data.is_empty() {
            stacktrace = Some(StackTrace::try_from(data)?);
        }

        Ok(Thread {
            thread_id,
            thread_name,
            stacktrace,
            state,
        })
    }
}

impl<'a> ThreadDump<'a> {
    pub const FORMAT: &'static [FormatItem<'static>] =
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]");
}

impl<'a> TryFrom<&'a str> for ThreadDump<'a> {
    type Error = Parse;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let (header, rest) = value
            .trim_start()
            .split_once("\n")
            .ok_or(Parse::ExpectedNewline)?;
        let snapshot: u8;
        let timestamp: i64;
        match header
            .split(" : ")
            .collect::<Vec<&str>>()
            .as_slice()
        {
            ["Thread dump", raw_snapshot, raw_timestamp] => {
                snapshot = raw_snapshot.parse().map_err(|_| Parse::DumpSnapshot)?;
                let time = PrimitiveDateTime::parse(raw_timestamp, ThreadDump::FORMAT)
                    .map_err(Parse::SnapshotTimestampParsing)?;
                timestamp = time
                    .to_unix_millis()
                    .ok_or(Parse::SnapshotTimestampConversion)?;
            }
            _ => return Err(Parse::ThreadDumpExtraction),
        };

        let mut dump = rest.split_inclusive('\n').peekable();
        let mut start = 0;
        let mut offset = 0;
        let mut threads = HashMap::new();
        while let Some(line) = dump.next() {
            if line.starts_with("\"") {
                offset += line.len();
                while let Some(line) = dump.next_if(|l| !l.trim().starts_with("\"")) {
                    offset += line.len();
                }

                let contents = &rest[start..start + offset];
                let thread = Thread::try_from(contents);
                match thread {
                    Ok(thread) => threads.insert(thread.thread_id, thread),
                    Err(e) => {
                        warn!("Error during parsing thread dump: {e:?}, thread: {contents}");
                        None
                    }
                };

                start += offset;
                offset = 0;
            } else {
                start += line.len();
            }
        }
        Ok(ThreadDump {
            threads,
            triggered_unix_ms: timestamp,
            snapshot,
        })
    }
}

impl<'a> Iterator for ThreadDumpStreamer<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.trim_ascii().is_empty() {
            return None;
        }

        let mut iter = self.0.split_inclusive(|c| *c == b'\n').peekable();
        let mut start = 0;
        let mut offset = 0;
        while let Some(line) = iter.next() {
            if !line.starts_with(b"Thread dump") {
                start += line.len();
                continue;
            }
            offset += line.len();
            while let Some(line) = iter.next_if(|l| !l.trim_ascii().starts_with(b"TriggeredTime")) {
                offset += line.len();
            }
            break;
        }
        let contents = &self.0[start..start + offset];
        self.0 = &self.0[start + offset..];
        std::str::from_utf8(contents).ok()
    }
}
