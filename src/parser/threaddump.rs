use time::{PrimitiveDateTime, format_description::FormatItem, macros::format_description};
use tracing::warn;

use crate::{
    error::threaddump::Parse,
    parser::{
        scanner::Scanner,
        stacktrace::{Object, Trace},
    },
    util::{self, ToUnixMillis},
};

#[derive(Debug)]
pub struct LockInfo<'a> {
    pub owner_id: u64,
    pub owner_name: Option<&'a [u8]>,
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
    pub name: Option<&'a [u8]>,
}

#[derive(Debug)]
pub struct ThreadDump<'a> {
    pub threads: Vec<Thread<'a>>,
    pub triggered: u64,
    pub snapshot: u8,
}

impl<'a> TryFrom<&'a [u8]> for LockInfo<'a> {
    type Error = Parse;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        scanner.skip_whitespace();
        scanner
            .expect(b"LockName: ")
            .map_err(|_| Parse::ExpectedLockName)?;
        let object = scanner.take_until(b" ").ok_or(Parse::ExpectedLockObject)?;
        let object = Object::try_from(object)?;

        scanner.skip_whitespace();
        scanner
            .expect(b"Owner Id: ")
            .map_err(|_| Parse::ExpectedOwnerId)?;

        let owner_id: u64 = scanner
            .take_until(b" ")
            .ok_or(Parse::ExpectedOwnerId)
            .and_then(|v| util::parse_u64(v).map_err(Parse::ThreadIdParse))?;

        scanner.skip_whitespace();
        scanner
            .expect(b"Owner Name: ")
            .map_err(|_| Parse::ExpectedOwnerName)?;
        let owner_name = scanner.remaining().trim_ascii_start();
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
    const PREAMBLE: &'static [u8] = b"Java.lang.Thread.State:";
}

impl<'a> TryFrom<&'a [u8]> for ThreadState<'a> {
    type Error = Parse;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        scanner.skip_whitespace();
        scanner
            .expect(ThreadState::PREAMBLE)
            .map_err(|_| Parse::ExpectedPreamble)?;
        scanner.skip_whitespace();

        if let Some(raw_state) = scanner.peek_until(b" waiting to lock ") {
            let raw_state = raw_state.trim_ascii();
            if raw_state != b"BLOCKED" {
                return Err(Parse::ExpectedLockObject);
            }
            Ok(ThreadState::BlockedToLock(None))
        } else if let Some(raw_state) = scanner.peek_until(b" on ") {
            match raw_state.trim_ascii() {
                b"WAITING" => {
                    _ = scanner.take_until(b"on").expect("SAFETY: CHECKED");
                    scanner.skip_whitespace();
                    let object = scanner.remaining();
                    let object = Object::try_from(object)?;
                    Ok(ThreadState::WaitingOn(object))
                }
                b"TIMED_WAITING" => {
                    _ = scanner.take_until(b"on").expect("SAFETY: CHECKED");
                    scanner.skip_whitespace();
                    let object = scanner.remaining();
                    let object = Object::try_from(object)?;
                    Ok(ThreadState::TimedWaitingOn(object))
                }
                _ => Err(Parse::ExpectedWaitObject),
            }
        } else {
            match scanner.remaining().trim_ascii() {
                b"NEW" => Ok(ThreadState::New),
                b"TERMINATED" => Ok(ThreadState::Terminated),
                b"RUNNABLE" => Ok(ThreadState::Runnable),
                b"TIMED_WAITING" => Ok(ThreadState::TimedWaiting),
                b"WAITING" => Ok(ThreadState::Waiting),
                _ => Err(Parse::UnexpectedThreadState),
            }
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for Thread<'a> {
    type Error = Parse;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        scanner.skip_whitespace();
        let thread_name = scanner
            .take_within(b"\"", b"\"")
            .map_err(|_| Parse::ThreadNameExtraction)?
            .trim_ascii();

        let thread_name = if thread_name.is_empty() {
            None
        } else {
            Some(thread_name)
        };

        scanner.skip_whitespace();
        scanner
            .expect(b"Id=")
            .map_err(|_| Parse::ThreadIdExtraction)?;

        let id = scanner
            .take_until_inclusive(b" ")
            .ok_or(Parse::ThreadIdExtraction)?;
        let thread_id: u64 = util::parse_u64(id).map_err(Parse::ThreadIdParse)?;
        scanner.skip_whitespace();

        if scanner.peek_until(b"\n").is_none() {
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
            .take_until(b"\n")
            .ok_or(Parse::ThreadHeaderExtraction)?;
        let state = ThreadState::try_from(header)?;
        scanner.skip_whitespace();
        let state = if scanner.peek_expect(b"LockName: ") {
            let lock_info = scanner
                .take_until(b"\n")
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

        let data = scanner.remaining().trim_ascii();
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

impl<'a> TryFrom<&'a [u8]> for ThreadDump<'a> {
    type Error = Parse;
    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        let header = scanner
            .take_until(b"\n")
            .ok_or(Parse::ExpectedNewline)?
            .trim_ascii_start();
        let snapshot: u8;
        let timestamp: u64;
        match header
            .splitn(3, |b| *b == b':')
            .map(|b| b.trim_ascii())
            .collect::<Vec<_>>()
            .as_slice()
        {
            [b"Thread dump", raw_snapshot, raw_timestamp] => {
                snapshot = util::parse_u32(raw_snapshot).map_err(|_| Parse::DumpSnapshot)? as u8;
                let time = PrimitiveDateTime::parse(
                    &String::from_utf8_lossy(*raw_timestamp),
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
        let mut dump = rest.split_inclusive(|b| *b == b'\n').peekable();
        let mut start = 0;
        let mut offset = 0;
        let mut threads = Vec::with_capacity(100);
        while let Some(line) = dump.next() {
            if line.starts_with(b"\"") {
                offset += line.len();
                while let Some(line) = dump.next_if(|l| !l.trim_ascii().starts_with(b"\"")) {
                    offset += line.len();
                }

                let contents = &rest[start..start + offset];
                let thread = Thread::try_from(contents);
                match thread {
                    Ok(thread) => threads.push(thread),
                    Err(e) => {
                        warn!(
                            "Error during parsing thread dump: {e:?}, thread: {}",
                            String::from_utf8_lossy(contents)
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
