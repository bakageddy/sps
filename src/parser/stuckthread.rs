use crate::{
    error::stuckthread::Parse,
    parser::{scanner::Scanner, stacktrace::Trace},
    util::{self, ToUnixMillis},
};

use time::{
    Date, Duration, PrimitiveDateTime, Time, UtcDateTime,
    macros::{datetime, format_description},
};

static DATE_FORMAT: &[time::format_description::FormatItem<'static>] =
    format_description!("[day]-[month]-[year]");
static TIME_FORMAT: &[time::format_description::FormatItem<'static>] =
    format_description!("[hour]:[minute]:[second].[subsecond]");

#[derive(Debug)]
pub struct StuckThread<'a>(pub Event<'a>);

#[derive(Debug)]
pub enum Event<'a> {
    Begin(Begin<'a>, Trace<'a>),
    End(End<'a>),
}

#[derive(Debug)]
pub struct Begin<'a> {
    pub start: u64,
    pub tid: u64,
    pub active_duration_ms: u32,
    pub active_monitor_count: u32,
    pub name: &'a [u8],
    pub request: &'a [u8],
}

#[derive(Debug)]
pub struct End<'a> {
    pub start: u64,
    pub tid: u64,
    pub active_duration_ms: u32,
    pub active_monitor_count: u32,
    pub name: &'a [u8],
}


impl<'a> TryFrom<&'a [u8]> for StuckThread<'a> {
    type Error = Parse;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        let meta = scanner
            .take_until(b"\n")
            .ok_or_else(|| Parse::MetaExtractionError)
            .and_then(|m| Event::try_from(m))?;
        if let Event::Begin(begin, _) = meta {
            let stacktrace =
                Trace::try_from(scanner.remaining()).map_err(Parse::StackParseError)?;
            Ok(StuckThread(Event::Begin(begin, stacktrace)))
        } else {
            Ok(StuckThread(meta))
        }
    }
}

impl<'a> Event<'a> {
    fn extract_bracket_groups(input: &[u8]) -> Result<Vec<&[u8]>, Parse> {
        let mut scanner = Scanner::new(input);
        let mut groups = Vec::with_capacity(8);
        while let Ok(group) = scanner.take_within(b"[", b"]") {
            groups.push(group);
        }
        Ok(groups)
    }

    fn parse_date_time(date: &'a [u8], time: &'a [u8]) -> Result<PrimitiveDateTime, Parse> {
        let time = String::from_utf8_lossy(time);
        let date = String::from_utf8_lossy(date);
        let time = Time::parse(&time, TIME_FORMAT).map_err(Parse::InvalidDateTimeFormat)?;

        let date = Date::parse(&date, DATE_FORMAT).map_err(Parse::InvalidDateTimeFormat)?;

        let datetime = PrimitiveDateTime::new(date, time);
        Ok(datetime)
    }
}

impl<'a> TryFrom<&'a [u8]> for Event<'a> {
    type Error = Parse;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        let header = scanner.take_until(b"::").ok_or(Parse::DoubleColonAbsent)?;

        let headers = Event::extract_bracket_groups(header)?;
        let message = Event::extract_bracket_groups(scanner.remaining())?;
        let [time, date, _, _, _] = headers.as_slice() else {
            return Err(Parse::IncorrectHeaderInfoCount {
                got: headers
                    .iter()
                    .map(|s| String::from_utf8_lossy(s).to_string())
                    .collect(),
                expected: 5,
            });
        };

        let start = Event::parse_date_time(*date, *time)?;
        let start = start.to_unix_millis().unwrap_or(0);

        let mut event = match message.as_slice() {
            [_, _, _, _, _, _, _] => {
                Event::Begin(Begin::try_from(message.as_slice())?, Default::default())
            }
            [_, _, _, _] | [_, _, _] => Event::End(End::try_from(message.as_slice())?),
            _ => {
                return Err(Parse::IncorrectMessageInfoCount {
                    got: message
                        .iter()
                        .map(|s| String::from_utf8_lossy(*s).to_string())
                        .collect(),
                    minimum_expected: 3,
                });
            }
        };

        match event {
            Event::Begin(ref mut b, _) => {
                b.start = start
                    .checked_sub(b.active_duration_ms as u64)
                    .ok_or(Parse::DurationOverflow)?;
            }
            Event::End(ref mut b) => {
                b.start = start
                    .checked_sub(b.active_duration_ms as u64)
                    .ok_or(Parse::DurationOverflow)?;
            }
        };
        Ok(event)
    }
}

impl<'a> TryFrom<&[&'a [u8]]> for Begin<'a> {
    type Error = Parse;

    fn try_from(value: &[&'a [u8]]) -> Result<Self, Self::Error> {
        let [
            name,
            tid,
            active_duration,
            _,
            request,
            _threshold,
            active_thread_count,
        ] = value
        else {
            return Err(Parse::IncorrectMessageInfoCount {
                got: value
                    .iter()
                    .map(|s| String::from_utf8_lossy(*s).to_string())
                    .collect(),
                minimum_expected: 7,
            });
        };

        let tid = util::parse_u64(*tid).map_err(|inner| Parse::InvalidThreadId {
            got: String::from_utf8_lossy(*tid).to_string(),
            inner,
        })?;
        let active_duration_ms =
            util::parse_comma_separated_u32(*active_duration).map_err(|inner| {
                Parse::InvalidActiveDuration {
                    got: String::from_utf8_lossy(*active_duration).to_string(),
                    inner,
                }
            })?;
        let active_monitor_count = util::parse_u32(*active_thread_count).map_err(|inner| {
            Parse::InvalidActiveThreadCount {
                got: String::from_utf8_lossy(*active_thread_count).to_string(),
                inner,
            }
        })?;
        Ok(Begin {
            tid,
            active_duration_ms,
            active_monitor_count,
            name,
            request,
            start: 0,
        })
    }
}

impl<'a> TryFrom<&[&'a [u8]]> for End<'a> {
    type Error = Parse;

    fn try_from(value: &[&'a [u8]]) -> Result<Self, Self::Error> {
        let mut active_thread_count = None;
        let name;
        let tid;
        let active_duration_ms;
        match value {
            [tn, ti, adm, atc] => {
                name = *tn;
                tid = *ti;
                active_duration_ms = *adm;
                active_thread_count = Some(*atc);
            }
            [tn, ti, ad] => {
                name = *tn;
                tid = *ti;
                active_duration_ms = *ad;
            }
            _ => {
                return Err(Parse::IncorrectMessageInfoCount {
                    got: value
                        .iter()
                        .map(|s| String::from_utf8_lossy(s).to_string())
                        .collect(),
                    minimum_expected: 3,
                });
            }
        }

        let tid = util::parse_u64(tid).map_err(|inner| Parse::InvalidThreadId {
            got: String::from_utf8_lossy(tid).to_string(),
            inner,
        })?;
        let active_duration_ms =
            util::parse_comma_separated_u32(active_duration_ms).map_err(|inner| {
                Parse::InvalidActiveDuration {
                    got: String::from_utf8_lossy(active_duration_ms).to_string(),
                    inner,
                }
            })?;

        let mut active_monitor_count = 0;
        if let Some(active_thread_count) = active_thread_count {
            active_monitor_count = util::parse_u32(active_thread_count).map_err(|inner| {
                Parse::InvalidActiveThreadCount {
                    got: String::from_utf8_lossy(active_thread_count).to_string(),
                    inner,
                }
            })?;
        }

        Ok(End {
            start: 0,
            name,
            tid,
            active_duration_ms,
            active_monitor_count,
        })
    }
}
