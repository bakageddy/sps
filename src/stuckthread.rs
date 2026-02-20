use time::{
    Date, Duration, PrimitiveDateTime, Time, UtcDateTime,
    error::{InvalidFormatDescription, Parse},
};

use crate::stacktrace::{StackTrace, StackTraceParseError};

#[derive(Debug, Default)]
pub struct StuckThread<'a> {
    pub st: StackTrace<'a>,
    pub meta: StuckThreadMeta<'a>,
}

#[derive(Debug)]
pub struct StuckThreadMeta<'a> {
    pub start: time::PrimitiveDateTime,
    pub thread_id: u32,
    pub thread_name: &'a str,
    pub request: &'a str,
    pub active_duration_ms: i64,
    pub active_monitor_count: usize,

    pub end: Option<PrimitiveDateTime>,
}

impl<'a> TryFrom<&'a str> for StuckThread<'a> {
    type Error = StuckThreadParserError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let (meta, rest) = value
            .split_once('\n')
            .ok_or(StuckThreadParserError::MetaExtractionError)?;
        let stuck_thread_meta = StuckThreadMeta::try_from(meta)
            .map_err(|e| StuckThreadParserError::MetaInfoError(e))?;
        let stacktrace =
            StackTrace::try_from(rest).map_err(|e| StuckThreadParserError::StackTrace(e))?;

        Ok(Self {
            st: stacktrace,
            meta: stuck_thread_meta,
        })
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
    ) -> Result<Vec<&'b str>, StuckThreadMetaParserError> {
        assert!(Self::STUCKTHREAD_MESSAGE_USEFUL_INFO == 6);
        assert!(Self::STUCKTHREAD_MESSAGE_TOTAL_INFO == 7);
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
}

impl<'a> Default for StuckThreadMeta<'a> {
    fn default() -> Self {
        let now = UtcDateTime::now();
        Self {
            start: now.date().with_time(now.time()),
            thread_id: Default::default(),
            thread_name: Default::default(),
            active_duration_ms: 0,
            active_monitor_count: 0,
            request: Default::default(),
            end: Default::default(),
        }
    }
}

impl<'a> TryFrom<&'a str> for StuckThreadMeta<'a> {
    type Error = StuckThreadMetaParserError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let (mut header, mut message) = value
            .split_once("::")
            .ok_or(StuckThreadMetaParserError::DoubleColonAbsent)?;

        let headers = StuckThreadMeta::extract_bracket_groups(&mut header)?;
        if headers.len() < StuckThreadMeta::STUCKTHREAD_HEADER_TOTAL_INFO {
            return Err(StuckThreadMetaParserError::IncorrectHeaderInfoCount {
                got: headers.len(),
            });
        }

        let message = StuckThreadMeta::extract_bracket_groups(&mut message)?;
        if message.len() < StuckThreadMeta::STUCKTHREAD_MESSAGE_TOTAL_INFO {
            return Err(StuckThreadMetaParserError::IncorrectMessageInfoCount {
                got: message.len(),
            });
        }

        let time = headers.get(0).unwrap();
        let date = headers.get(1).unwrap();

        let date_format = time::format_description::parse("[day]-[month]-[year]")
            .map_err(|e| StuckThreadMetaParserError::InvalidTimeFormatDescription(e))?;
        let time_format =
            time::format_description::parse("[hour]:[minute]:[second].[subsecond]")
                .map_err(|e| StuckThreadMetaParserError::InvalidDateFormatDescription(e))?;

        let time = Time::parse(time, &time_format)
            .map_err(|e| StuckThreadMetaParserError::InvalidTimeFormat(e))?;

        let date = Date::parse(date, &date_format)
            .map_err(|e| StuckThreadMetaParserError::InvalidDateFormat(e))?;

        let start = PrimitiveDateTime::new(date, time);
        let thread_name = message.get(0).expect("IncorrectMessageInfoCount");

        let thread_id = message.get(1).expect("IncorrectMessageInfoCount");
        let thread_id =
            thread_id
                .parse::<u32>()
                .map_err(|_| StuckThreadMetaParserError::InvalidThreadId {
                    got: thread_id.to_string(),
                })?;

        let active_duration = message.get(2).expect("IncorrectMessageInfoCount");
        let active_duration = active_duration.replace(",", "");
        let active_duration = active_duration.parse::<i64>().map_err(|_| {
            StuckThreadMetaParserError::InvalidDuration {
                got: active_duration,
            }
        })?;
        let api_request = message.get(4).expect("IncorrectMessageInfoCount");
        let active_thread_count = message.get(6).expect("IncorrectMessageInfoCount");
        let active_thread_count = active_thread_count.parse::<usize>().map_err(|_| {
            StuckThreadMetaParserError::InvalidActiveThreadCount {
                got: active_thread_count.to_string(),
            }
        })?;

        let start = start
            .checked_sub(Duration::milliseconds(active_duration))
            .ok_or(StuckThreadMetaParserError::DurationOverflow)?;
        Ok(StuckThreadMeta {
            start,
            thread_id,
            thread_name: *thread_name,
            request: *api_request,
            active_duration_ms: active_duration,
            active_monitor_count: active_thread_count,
            end: None,
        })
    }
}

#[derive(Debug)]
pub enum StuckThreadParserError {
    MetaInfoError(StuckThreadMetaParserError),
    MetaExtractionError,

    StackTrace(StackTraceParseError),
}

#[derive(Debug)]
pub enum StuckThreadMetaParserError {
    DoubleColonAbsent,
    UnmatchedRightBracket(usize),

    IncorrectHeaderInfoCount { got: usize },
    IncorrectMessageInfoCount { got: usize },

    InvalidTimeFormat(Parse),
    InvalidDateFormat(Parse),

    InvalidTimeFormatDescription(InvalidFormatDescription),
    InvalidDateFormatDescription(InvalidFormatDescription),

    InvalidDuration { got: String },
    InvalidThreadId { got: String },
    InvalidActiveThreadCount { got: String },

    DurationOverflow,
}
