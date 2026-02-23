use std::{fs, path::Path};

use time::{
    Date, Duration, PrimitiveDateTime, Time,
    error::{InvalidFormatDescription, Parse},
    format_description::FormatItem,
    macros::{datetime, format_description},
};

use crate::stacktrace::{StackTrace, StackTraceParseError};

static DATE_FORMAT: &[FormatItem<'static>] = format_description!("[day]-[month]-[year]");
static TIME_FORMAT: &[FormatItem<'static>] =
    format_description!("[hour]:[minute]:[second].[subsecond]");

pub trait ToUnixMillis {
    fn to_unix_millis(&self) -> Option<i64>;
}

#[derive(Debug)]
pub struct StuckThread<'a> {
    pub st: Option<StackTrace<'a>>,
    pub meta: StuckThreadMeta<'a>,
}

#[derive(Debug)]
pub enum StuckThreadMeta<'a> {
    Begin(StuckThreadMetaBegin<'a>),
    End(StuckThreadMetaEnd<'a>),
}

#[derive(Debug)]
pub struct StuckThreadMetaBegin<'a> {
    pub start: time::PrimitiveDateTime,
    pub thread_id: u32,
    pub thread_name: &'a str,
    pub request: &'a str,
    pub active_duration_ms: i64,
    pub active_monitor_count: i64,
}

#[derive(Debug)]
pub struct StuckThreadMetaEnd<'a> {
    pub thread_name: &'a str,
    pub thread_id: u32,
    pub active_duration_ms: i64,
    pub active_monitor_count: i64,
}

impl ToUnixMillis for PrimitiveDateTime {
    fn to_unix_millis(&self) -> Option<i64> {
        let result = self.assume_utc().unix_timestamp_nanos() / 1_000_000;
        i64::try_from(result).ok()
    }
}

pub struct StuckThreadProducer;

impl StuckThreadProducer {
    pub fn produce<'a>(
        contents: &'a str,
    ) -> Option<Vec<Result<StuckThread<'a>, StuckThreadParserError<'a>>>> {

        let mut result = vec![];

        let mut start = 0;
        let mut offset = 0;
        for line in contents.split_inclusive('\n') {
            if line.starts_with('[') {
                let record = &contents[start..start + offset];
                result.push(StuckThread::try_from(record));

                start = start + offset;
                offset = line.len();
            } else {
                offset += line.len();
            }
        }

        Some(result)
    }
}

impl<'a> TryFrom<&'a str> for StuckThread<'a> {
    type Error = StuckThreadParserError<'a>;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let (meta, rest) = value
            .split_once('\n')
            .ok_or(StuckThreadParserError::MetaExtractionError)?;
        let stuck_thread_meta = StuckThreadMeta::try_from(meta)
            .map_err(|e| StuckThreadParserError::MetaInfoError(e))?;

        match stuck_thread_meta {
            StuckThreadMeta::Begin(_) => {
                let stacktrace = StackTrace::try_from(rest)
                    .map_err(|e| StuckThreadParserError::StackTrace(e))?;
                return Ok(Self {
                    st: Some(stacktrace),
                    meta: stuck_thread_meta,
                });
            }

            StuckThreadMeta::End(_) => {
                return Ok(Self {
                    st: None,
                    meta: stuck_thread_meta,
                });
            }
        }
    }
}

impl<'a> StuckThreadMeta<'a> {
    const STUCKTHREAD_TOTAL_INFO: usize = 12;
    const STUCKTHREAD_USEFUL_INFO: usize = 8;

    const STUCKTHREAD_HEADER_USEFUL_INFO: usize = 2;
    const STUCKTHREAD_HEADER_TOTAL_INFO: usize = 5;

    const STUCKTHREAD_MESSAGE_USEFUL_INFO: usize = 6;
    const STUCKTHREAD_MESSAGE_TOTAL_INFO: usize = 7;

    fn extract_bracket_groups<'b>(
        input: &mut &'b str,
    ) -> Result<Vec<&'b str>, StuckThreadMetaParserError<'b>> {
        let mut groups = Vec::with_capacity(8);
        let iter_count: usize = 0;
        while let Some((_, rest)) = input.split_once('[') {
            let (group, rest) =
                rest.split_once(']')
                    .ok_or(StuckThreadMetaParserError::UnmatchedRightBracket(
                        iter_count,
                    ))?;

            groups.push(group);
            *input = rest;
        }
        Ok(groups)
    }

    fn parse_thread_id(value: &'a str) -> Option<u32> {
        value.parse::<u32>().ok()
    }

    fn parse_comma_separate_i64(value: &'a str) -> Option<i64> {
        value.replace(",", "").parse::<i64>().ok()
    }

    fn parse_i64(value: &'a str) -> Option<i64> {
        value.parse::<i64>().ok()
    }

    fn parse_date_time(
        date: &'a str,
        time: &'a str,
    ) -> Result<PrimitiveDateTime, StuckThreadMetaParserError<'a>> {
        let time = Time::parse(time, TIME_FORMAT)
            .map_err(|e| StuckThreadMetaParserError::InvalidTimeFormat(e))?;

        let date = Date::parse(date, DATE_FORMAT)
            .map_err(|e| StuckThreadMetaParserError::InvalidDateFormat(e))?;

        let datetime = PrimitiveDateTime::new(date, time);
        Ok(datetime)
    }
}

impl<'a> TryFrom<&'a str> for StuckThreadMeta<'a> {
    type Error = StuckThreadMetaParserError<'a>;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let (mut header, mut message) = value
            .split_once("::")
            .ok_or(StuckThreadMetaParserError::DoubleColonAbsent)?;

        let headers = StuckThreadMeta::extract_bracket_groups(&mut header)?;
        let message = StuckThreadMeta::extract_bracket_groups(&mut message)?;

        let [time, date, _, _, _] = headers.as_slice() else {
            return Err(StuckThreadMetaParserError::IncorrectHeaderInfoCount {
                got: headers.len(),
                groups: headers,
            });
        };

        let start = StuckThreadMeta::parse_date_time(date, time)?;

        let mut meta: StuckThreadMeta;
        let message_len = message.len();
        if message_len == 7 {
            meta = StuckThreadMeta::Begin(StuckThreadMetaBegin::try_from(message)?);
        } else if message_len == 4 || message_len == 3 {
            meta = StuckThreadMeta::End(StuckThreadMetaEnd::try_from(message)?);
        } else {
            return Err(StuckThreadMetaParserError::IncorrectMessageInfoCount {
                got: message.len(),
                groups: message,
            });
        }

        match meta {
            StuckThreadMeta::Begin(ref mut b) => {
                b.start = start
                    .checked_sub(Duration::milliseconds(b.active_duration_ms))
                    .ok_or(StuckThreadMetaParserError::DurationOverflow)?;
            }
            _ => {}
        };

        Ok(meta)
    }
}

impl<'a> TryFrom<Vec<&'a str>> for StuckThreadMetaBegin<'a> {
    type Error = StuckThreadMetaParserError<'a>;

    fn try_from(value: Vec<&'a str>) -> Result<Self, Self::Error> {
        if value.len() != 7 {
            return Err(StuckThreadMetaParserError::IncorrectHeaderInfoCount {
                got: value.len(),
                groups: value,
            });
        }

        let [
            thread_name,
            thread_id,
            active_duration,
            _,
            api_request,
            _threshold,
            active_thread_count,
        ] = value.as_slice()
        else {
            return Err(StuckThreadMetaParserError::IncorrectMessageInfoCount {
                got: value.len(),
                groups: value,
            });
        };

        let thread_id = StuckThreadMeta::parse_thread_id(*thread_id)
            .ok_or(StuckThreadMetaParserError::InvalidThreadId { got: *thread_id })?;

        let active_duration = StuckThreadMeta::parse_comma_separate_i64(*active_duration).ok_or(
            StuckThreadMetaParserError::InvalidActiveDuration {
                got: *active_duration,
            },
        )?;

        let active_thread_count = StuckThreadMeta::parse_i64(*active_thread_count).ok_or(
            StuckThreadMetaParserError::InvalidActiveThreadCount {
                got: *active_thread_count,
            },
        )?;

        Ok(StuckThreadMetaBegin {
            start: datetime!(1970-01-01 00:00:00.0),
            active_duration_ms: active_duration,
            active_monitor_count: active_thread_count,
            thread_id,
            thread_name: *thread_name,
            request: *api_request,
        })
    }
}

impl<'a> TryFrom<Vec<&'a str>> for StuckThreadMetaEnd<'a> {
    type Error = StuckThreadMetaParserError<'a>;

    fn try_from(value: Vec<&'a str>) -> Result<Self, Self::Error> {
        let mut active_thread_count = None;
        let thread_name: &str;
        let thread_id: &str;
        let active_duration: &str;

        match value.as_slice() {
            [tn, ti, ad] => {
                thread_name = *tn;
                thread_id = *ti;
                active_duration = *ad;
            },

            [tn, ti, ad, atc] => {
                thread_name = *tn;
                thread_id = *ti;
                active_duration = *ad;
                active_thread_count = Some(atc);
            },

            _ => {
                return Err(StuckThreadMetaParserError::IncorrectMessageInfoCount {
                    got: value.len(),
                    groups: value,
                });
            }
        }

        let thread_id = StuckThreadMeta::parse_thread_id(thread_id)
            .ok_or(StuckThreadMetaParserError::InvalidThreadId { got: thread_id })?;

        let active_duration = StuckThreadMeta::parse_comma_separate_i64(active_duration).ok_or(
            StuckThreadMetaParserError::InvalidActiveDuration {
                got: active_duration,
            },
        )?;

        let mut active_monitor_count = 0;
        if let Some(active_thread_count) = active_thread_count {
            active_monitor_count = StuckThreadMeta::parse_i64(active_thread_count).ok_or(
                StuckThreadMetaParserError::InvalidActiveThreadCount {
                    got: active_thread_count,
                },
            )?;
        }

        Ok(StuckThreadMetaEnd {
            thread_name: thread_name,
            thread_id,
            active_duration_ms: active_duration,
            active_monitor_count: active_monitor_count,
        })
    }
}

#[derive(Debug)]
pub enum StuckThreadParserError<'a> {
    MetaInfoError(StuckThreadMetaParserError<'a>),
    MetaExtractionError,

    StackTrace(StackTraceParseError),
}

#[derive(Debug)]
pub enum StuckThreadMetaParserError<'a> {
    DoubleColonAbsent,
    UnmatchedRightBracket(usize),

    IncorrectHeaderInfoCount { got: usize, groups: Vec<&'a str> },
    IncorrectMessageInfoCount { got: usize, groups: Vec<&'a str> },

    InvalidTimeFormat(Parse),
    InvalidDateFormat(Parse),

    InvalidTimeFormatDescription(InvalidFormatDescription),
    InvalidDateFormatDescription(InvalidFormatDescription),

    InvalidThreadId { got: &'a str },
    InvalidActiveThreadCount { got: &'a str },
    InvalidActiveDuration { got: &'a str },

    DurationOverflow,
}
