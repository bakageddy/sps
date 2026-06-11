use time::{PrimitiveDateTime, format_description::FormatItem, macros::format_description};
use tracing::warn;

use crate::{
    error::threaddump::Parse,
    parser::{
        scanner::Scanner,
        stacktrace::{Object, Trace},
    },
    util::ToUnixMillis,
};

#[derive(Debug)]
pub struct LockInfo<'a> {
    pub owner_id: u64,
    pub owner_name: Option<&'a str>,
    pub object: Object<'a>,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Thread<'a> {
    pub tid: u64,
    pub state: ThreadState<'a>,
    pub stacktrace: Option<Trace<'a>>,
    pub name: Option<&'a str>,
}

#[derive(Debug)]
pub struct ThreadDump<'a> {
    pub threads: Vec<Thread<'a>>,
    pub triggered: u64,
    pub snapshot: u8,
}

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

        let owner_id: u64 = scanner
            .take_until(" ")
            .ok_or(Parse::ExpectedOwnerId)
            .and_then(|v| v.parse().map_err(Parse::ThreadIdParse))?;

        scanner.skip_whitespace();
        scanner
            .expect("Owner Name: ")
            .map_err(|_| Parse::ExpectedOwnerName)?;

        let owner_name = scanner.remaining();
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
            if raw_state != "BLOCKED" {
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
                _ => Err(Parse::ExpectedWaitObject),
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
            .trim_ascii();

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
        let thread_id: u64 = id.parse().map_err(Parse::ThreadIdParse)?;
        scanner.skip_whitespace();

        if scanner.peek_until("\n").is_none() {
            let header = scanner.remaining();
            let state = ThreadState::try_from(header)?;
            return Ok(Thread {
                tid: thread_id,
                state,
                name: thread_name,
                stacktrace: None,
            });
        }

        let header = scanner
            .take_until("\n")
            .ok_or(Parse::ThreadHeaderExtraction)?;
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
        let mut stacktrace: Option<Trace<'a>> = None;
        if !data.is_empty() {
            stacktrace = Some(Trace::try_from(data)?);
        }

        Ok(Thread {
            tid: thread_id,
            name: thread_name,
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
        let mut scanner = Scanner::new(value);
        let header = scanner
            .take_until("\n")
            .ok_or(Parse::ExpectedNewline)?
            .trim_start();
        let snapshot: u8;
        let timestamp: u64;
        match header
            .splitn(3, ":")
            .map(|b| b.trim())
            .collect::<Vec<_>>()
            .as_slice()
        {
            ["Thread dump", raw_snapshot, raw_timestamp] => {
                snapshot = (*raw_snapshot).parse().map_err(|_| Parse::DumpSnapshot)?;
                let time = PrimitiveDateTime::parse(
                    *raw_timestamp,
                    ThreadDump::FORMAT,
                )
                .map_err(Parse::SnapshotTimestampParsing)?;
                timestamp = time
                    .to_unix_millis()
                    .ok_or(Parse::SnapshotTimestampConversion)?;
            }
            _ => return Err(Parse::ThreadDumpExtraction),
        };

        let rest = scanner.remaining();
        let mut dump = rest.split_inclusive("\n").peekable();
        let mut start = 0;
        let mut offset = 0;
        let mut threads = Vec::with_capacity(100);
        while let Some(line) = dump.next() {
            if line.starts_with("\"") {
                offset += line.len();
                while let Some(line) = dump.next_if(|l| !l.trim().starts_with("\"")) {
                    offset += line.len();
                }

                let contents = &rest[start..start + offset];
                let thread = Thread::try_from(contents);
                match thread {
                    Ok(thread) => threads.push(thread),
                    Err(e) => {
                        warn!(
                            "Error during parsing thread dump: {e:?}, thread: {}",
                            String::from(contents)
                        );
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
            triggered: timestamp,
            snapshot,
        })
    }
}
