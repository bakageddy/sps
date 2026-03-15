use std::collections::HashMap;

use crate::{error::threaddump::Parse, scanner::Scanner};

#[derive(Debug, PartialEq, PartialOrd, Eq, Default)]
pub struct Object<'a> {
    pub class: &'a str,
    pub identity: u64,
}

#[derive(Debug, PartialEq, PartialOrd, Eq)]
pub enum Source<'a> {
    NativeMethod,
    UnknownSource,
    Generated(&'a str),
    Filename { file: &'a str, line_number: i64 },
}

#[derive(Debug, PartialEq, PartialOrd, Eq)]
pub enum Element<'a> {
    Lock(Object<'a>),
    Elem { frame: &'a str, source: Source<'a> },
}

#[derive(Debug, PartialEq, PartialOrd, Eq)]
pub struct StackTrace<'a> {
    pub elem: Vec<Element<'a>>,
}

#[derive(Debug, PartialEq, PartialOrd, Eq)]
pub enum ThreadState<'a> {
    New,
    Terminated,
    Runnable,
    Blocked {
        owner_id: ThreadID,
        owner_name: Option<&'a str>,
        object: Object<'a>,
    },
    TimedWaiting(Object<'a>),
    Waiting(Object<'a>),
}

#[derive(Debug, PartialEq, PartialOrd, Eq)]
pub struct ThreadHeader<'a> {
    thread_name: Option<&'a str>,
    thread_id: ThreadID,
    state: ThreadState<'a>,
}

pub type ThreadID = i64;

#[derive(Debug, PartialEq, PartialOrd, Eq)]
pub struct Thread<'a> {
    pub thread_id: ThreadID,
    pub state: ThreadState<'a>,
    pub stacktrace: Option<StackTrace<'a>>,
    pub thread_name: Option<&'a str>,
}

pub struct ThreadDump<'a> {
    pub threads: HashMap<ThreadID, Thread<'a>>,
    pub triggered_unix_ms: i64,
}

impl Object<'_> {
    pub fn hex_to_u64(value: &str) -> Result<u64, Parse> {
        let value = value.trim();
        let mut result = 0;
        let mut i = 1;
        for char in value.chars().rev() {
            let weight = match char {
                '0' => 0,
                '1' => 1,
                '2' => 2,
                '3' => 3,
                '4' => 4,
                '5' => 5,
                '6' => 6,
                '7' => 7,
                '8' => 8,
                '9' => 9,
                'a' => 10,
                'b' => 11,
                'c' => 12,
                'd' => 13,
                'e' => 14,
                'f' => 15,
                c => return Err(Parse::HexUnexpectedChar { got: c }),
            };
            result += weight * i;
            i *= 16;
        }

        Ok(result)
    }
}

impl<'a> TryFrom<&'a str> for Object<'a> {
    type Error = Parse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        scanner.skip_whitespace();
        let class = scanner.take_until("@").ok_or(Parse::MissingCommat)?;
        Ok(Object {
            class,
            identity: Object::hex_to_u64(scanner.remaining())?,
        })
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

        if value.contains("$") {
            return Ok(Source::Generated(value));
        }

        let (file, lineno) = value.split_once(":").ok_or(Parse::ColonNotFound)?;
        return Ok(Source::Filename {
            file,
            line_number: lineno.parse::<i64>()?,
        });
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
            .map_err(|e| Parse::CloseParenNotFound)?;
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

        Ok(StackTrace { elem: frames })
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
            .map_err(|e| Parse::ExpectedPreamble)?;
        scanner.skip_whitespace();

        let mut state: &'a str;

        if let Some(raw_state) = scanner.peek_until("on") {
            match raw_state.trim() {
                "WAITING" => {
                    _ = scanner.take_until("on").expect("SAFETY: CHECKED");
                    scanner.skip_whitespace();
                    let object = scanner.remaining();
                    let object = Object::try_from(object)?;
                    return Ok(ThreadState::Waiting(object));
                }
                "TIMED_WAITING" => {
                    _ = scanner.take_until("on").expect("SAFETY: CHECKED");
                    scanner.skip_whitespace();
                    let object = scanner.remaining();
                    let object = Object::try_from(object)?;
                    return Ok(ThreadState::TimedWaiting(object));
                }
                _ => return Err(Parse::UnexpectedThreadState),
            };
        } else if let Some(raw_state) = scanner.peek_until("waiting to lock") {
            let raw_state = raw_state.trim();
            if raw_state.trim() != "BLOCKED" {
                return Err(Parse::UnexpectedThreadState);
            }
            return Ok(ThreadState::Blocked {
                owner_id: 0,
                owner_name: None,
                object: Object::default(),
            });
        } else {
            return match scanner.remaining().trim() {
                "NEW" => Ok(ThreadState::New),
                "TERMINATED" => Ok(ThreadState::Terminated),
                "RUNNABLE" => Ok(ThreadState::Runnable),
                _ => Err(Parse::UnexpectedThreadState),
            };
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
        let thread_id: i64 = id.parse().map_err(|e| Parse::ThreadIdParse(e))?;
        scanner.skip_whitespace();

        let header = scanner
            .take_until("\n")
            .ok_or(Parse::ThreadHeaderExtraction)?;
        let state = ThreadState::try_from(header)?;
        let state = match state {
            ThreadState::Blocked {
                owner_id,
                owner_name,
                object,
            } => {
                scanner.skip_whitespace();
                scanner
                    .expect("LockName: ")
                    .map_err(|_| Parse::ExpectedLockName)?;
                let object = scanner.take_until(" ").ok_or(Parse::ExpectedLockName)?;
                let object = Object::try_from(object)?;

                scanner.skip_whitespace();
                scanner
                    .expect("Owner Id: ")
                    .map_err(|_| Parse::ExpectedOwnerId)?;
                let owner_id = scanner.take_until(" ").ok_or(Parse::ExpectedOwnerId)?;
                let owner_id: i64 = owner_id.parse().map_err(|e| Parse::ThreadIdParse(e))?;

                scanner.skip_whitespace();
                scanner
                    .expect("Owner Name: ")
                    .map_err(|_| Parse::ExpectedOwnerName)?;
                let owner_name = scanner.take_until("\n").ok_or(Parse::ExpectedNewline)?;
                let owner_name = if owner_name.trim().is_empty() {
                    None
                } else {
                    Some(owner_name.trim())
                };
                ThreadState::Blocked {
                    owner_id,
                    owner_name,
                    object,
                }
            }
            _ => state,
        };

        let data = scanner.remaining();
        let mut stack: Option<StackTrace<'a>> = None;
        if !data.is_empty() {
            stack = Some(StackTrace::try_from(data)?);
        }

        Ok(Thread {
            thread_id,
            thread_name,
            stacktrace: stack,
            state,
        })
    }
}

impl<'a> TryFrom<&'a str> for ThreadDump<'a> {
    type Error = Parse;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        todo!()
    }
}
