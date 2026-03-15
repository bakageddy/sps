use time::{
    Date, Duration, PrimitiveDateTime, Time,
    macros::{datetime, format_description},
};

use crate::{
    error::stuckthread::{Error, Parse},
    scanner::Scanner,
    stacktrace::StackTrace,
};

static DATE_FORMAT: &[time::format_description::FormatItem<'static>] =
    format_description!("[day]-[month]-[year]");
static TIME_FORMAT: &[time::format_description::FormatItem<'static>] =
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
    pub start: time::PrimitiveDateTime,
    pub thread_id: u32,
    pub thread_name: &'a str,
    pub active_duration_ms: i64,
    pub active_monitor_count: i64,
}

impl ToUnixMillis for PrimitiveDateTime {
    fn to_unix_millis(&self) -> Option<i64> {
        let result = self.assume_utc().unix_timestamp_nanos() / 1_000_000;
        i64::try_from(result).ok()
    }
}

pub struct StuckThreadStream;

impl StuckThreadStream {
    pub fn parse<'a>(contents: &'a [u8]) -> Result<Vec<Result<StuckThread<'a>, Parse>>, Error> {
        let contents = str::from_utf8(contents)?;
        let mut iter = contents.split_inclusive('\n').into_iter().peekable();

        let mut lno = 0;
        let mut start = 0;
        let mut offset = 0;

        let mut output = Vec::new();
        while let Some(line) = iter.next() {
            lno += 1;
            if line.starts_with('[') {
                offset += line.len();
                while let Some(line) = iter.next_if(|s| !s.starts_with('[')) {
                    offset += line.len();
                    lno += 1;
                }
                let record = &contents[start..start + offset];
                let result = StuckThread::try_from(record);
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error during parsing around line {lno} {e:?}");
                    }
                }

                output.push(StuckThread::try_from(record));

                start = start + offset;
                offset = 0;
            }
        }
        Ok(output)
    }
}

impl<'a> TryFrom<&'a str> for StuckThread<'a> {
    type Error = Parse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);

        let meta = scanner
            .take_until("\n")
            .ok_or(Parse::MetaExtractionError)?;

        let stuck_thread_meta = StuckThreadMeta::try_from(meta)?;
        match stuck_thread_meta {
            StuckThreadMeta::Begin(_) => {
                let stacktrace =
                    StackTrace::try_from(scanner.remaining()).map_err(|e| Parse::StackParseError(e))?;

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
    fn extract_bracket_groups<'b>(input: &'b str) -> Result<Vec<&'b str>, Parse> {
        let mut scanner = Scanner::new(input);
        let mut groups = Vec::with_capacity(8);
        while let Ok(group) = scanner.take_within("[", "]") {
            groups.push(group);
        }
        Ok(groups)
    }

    fn parse_thread_id(value: &'a str) -> Result<u32, Parse> {
        value.parse::<u32>().map_err(|e| Parse::InvalidThreadId {
            got: value.to_string(),
            inner: e,
        })
    }

    fn parse_comma_separate_i64(value: &'a str) -> Result<i64, Parse> {
        value
            .replace(",", "")
            .parse::<i64>()
            .map_err(|e| Parse::InvalidActiveDuration {
                got: value.to_string(),
                inner: e,
            })
    }

    fn parse_i64(value: &'a str) -> Result<i64, Parse> {
        value
            .parse::<i64>()
            .map_err(|e| Parse::InvalidActiveThreadCount {
                got: value.to_string(),
                inner: e,
            })
    }

    fn parse_date_time(date: &'a str, time: &'a str) -> Result<PrimitiveDateTime, Parse> {
        let time = Time::parse(time, TIME_FORMAT).map_err(|e| Parse::InvalidDateTimeFormat(e))?;

        let date = Date::parse(date, DATE_FORMAT).map_err(|e| Parse::InvalidDateTimeFormat(e))?;

        let datetime = PrimitiveDateTime::new(date, time);
        Ok(datetime)
    }
}

impl<'a> TryFrom<&'a str> for StuckThreadMeta<'a> {
    type Error = Parse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        let header = scanner
            .take_until("::")
            .ok_or(Parse::DoubleColonAbsent)?;

        let headers = StuckThreadMeta::extract_bracket_groups(header)?;
        let message = StuckThreadMeta::extract_bracket_groups(scanner.remaining())?;

        let [time, date, _, _, _] = headers.as_slice() else {
            return Err(Parse::IncorrectHeaderInfoCount {
                got: headers.iter().map(|s| (*s).to_string()).collect(),
                expected: 5,
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
            return Err(Parse::IncorrectMessageInfoCount {
                got: message.iter().map(|s| (*s).to_string()).collect(),
                minimum_expected: 3,
            });
        }

        match meta {
            StuckThreadMeta::Begin(ref mut b) => {
                b.start = start
                    .checked_sub(Duration::milliseconds(b.active_duration_ms))
                    .ok_or(Parse::DurationOverflow)?;
            }
            StuckThreadMeta::End(ref mut b) => {
                b.start = start
                    .checked_sub(Duration::milliseconds(b.active_duration_ms))
                    .ok_or(Parse::DurationOverflow)?;
            }
        };

        Ok(meta)
    }
}

impl<'a> TryFrom<Vec<&'a str>> for StuckThreadMetaBegin<'a> {
    type Error = Parse;

    fn try_from(value: Vec<&'a str>) -> Result<Self, Self::Error> {
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
            return Err(Parse::IncorrectMessageInfoCount {
                got: value.iter().map(|s| s.to_string()).collect(),
                minimum_expected: 7,
            });
        };

        let thread_id = StuckThreadMeta::parse_thread_id(*thread_id)?;
        let active_duration = StuckThreadMeta::parse_comma_separate_i64(*active_duration)?;
        let active_thread_count = StuckThreadMeta::parse_i64(*active_thread_count)?;

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
    type Error = Parse;

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
            }

            [tn, ti, ad, atc] => {
                thread_name = *tn;
                thread_id = *ti;
                active_duration = *ad;
                active_thread_count = Some(atc);
            }

            _ => {
                return Err(Parse::IncorrectMessageInfoCount {
                    got: value.iter().map(|s| s.to_string()).collect(),
                    minimum_expected: 3,
                });
            }
        }

        let thread_id = StuckThreadMeta::parse_thread_id(thread_id)?;
        let active_duration = StuckThreadMeta::parse_comma_separate_i64(active_duration)?;
        let mut active_monitor_count = 0;
        if let Some(active_thread_count) = active_thread_count {
            active_monitor_count = StuckThreadMeta::parse_i64(active_thread_count)?;
        }

        Ok(StuckThreadMetaEnd {
            start: datetime!(1970-01-01 00:00:00.0),
            thread_name: thread_name,
            thread_id,
            active_duration_ms: active_duration,
            active_monitor_count: active_monitor_count,
        })
    }
}
