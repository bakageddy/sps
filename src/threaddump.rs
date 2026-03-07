use std::collections::HashMap;

use crate::error::threaddump::Parse;

#[derive(Debug, PartialEq, PartialOrd, Eq)]
pub struct Object<'a> {
    pub class: &'a str,
    pub object_id: u64,
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
    Blocked(Object<'a>),
    TimedWaiting(Object<'a>),
    Waiting(Object<'a>),
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
}

impl Object<'_> {
    pub fn hex_to_u64(value: &str) -> Result<u64, Parse> {
        let value = value.trim();
        if value.len() != 8 {
            return Err(Parse::HexLen {
                expected: 8,
                got: value.len(),
            });
        }
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
        let value = value.trim_start();
        match value.split_once('@') {
            Some((class, object_id)) => Ok(Object {
                class,
                object_id: Object::hex_to_u64(object_id)?,
            }),
            None => return Err(Parse::MissingCommat),
        }
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
        let value = value.trim();
        if value.starts_with("- locked ") {
            let value = value
                .strip_prefix("- locked ")
                .ok_or(Parse::LockedNotFound)?;
            let result = Object::try_from(value)?;
            return Ok(Element::Lock(result));
        }

        let (frame, rest) = value.split_once("(").ok_or(Parse::OpenParenNotFound)?;
        let (source, _) = rest.split_once(")").ok_or(Parse::CloseParenNotFound)?;

        let source = Source::try_from(source)?;
        return Ok(Element::Elem { frame, source });
    }
}

impl<'a> TryFrom<&'a str> for StackTrace<'a> {
    type Error = Parse;
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
        let value = value.trim();
        if !value.starts_with(ThreadState::PREAMBLE) {
            return Err(Parse::UnexpectedPreamble);
        }

        let value = value
            .trim()
            .strip_prefix(ThreadState::PREAMBLE)
            .expect("SAFETY: checked")
            .trim();

        match value.split_once("on") {
            Some((state, object)) => {
                let state = state.trim();
                let object = Object::try_from(object)?;
                return match state {
                    "WAITING" => Ok(ThreadState::Waiting(object)),
                    "TIMED_WAITING" => Ok(ThreadState::TimedWaiting(object)),
                    "BLOCKED" => Ok(ThreadState::Blocked(object)),
                    _ => Err(Parse::UnexpectedThreadState),
                };
            }
            None => {
                return match value {
                    "NEW" => Ok(ThreadState::New),
                    "TERMINATED" => Ok(ThreadState::Terminated),
                    "RUNNABLE" => Ok(ThreadState::Runnable),
                    _ => Err(Parse::UnexpectedThreadState),
                };
            }
        }
    }
}

impl<'a> TryFrom<&'a str> for Thread<'a> {
    type Error = Parse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let (value, stack) = value.split_once("\n").ok_or(Parse::UnexpectedPreamble)?;
        let (_, rest) = value.split_once("\"").ok_or(Parse::DoubleQuoteNotFound)?;
        let (name, rest) = rest.split_once("\"").ok_or(Parse::DoubleQuoteNotFound)?;

        let thread_name = if name.is_empty() { None } else { Some(name) };

        let rest = rest.trim();
        let (id, rest) = rest.split_once(" ").ok_or(Parse::ThreadIDExtraction)?;

        let (_, id) = id.split_once("=").ok_or(Parse::EqualsNotFound)?;
        let thread_id = id.parse::<i64>().map_err(|e| Parse::ThreadIdParse(e))?;

        let state = ThreadState::try_from(rest)?;
        let stacktrace = if stack.is_empty() {
            None
        } else {
            Some(StackTrace::try_from(stack)?)
        };

        Ok(Thread {
            thread_id,
            thread_name,
            stacktrace,
            state,
        })
    }
}
