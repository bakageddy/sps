// use time::{
//     Date, Duration, PrimitiveDateTime, Time,
//     macros::{datetime, format_description},
// };
//
// use crate::{error::stuckthread::Parse, scanner::Scanner, stacktrace::StackTrace};
//
// static DATE_FORMAT: &[time::format_description::FormatItem<'static>] =
//     format_description!("[day]-[month]-[year]");
// static TIME_FORMAT: &[time::format_description::FormatItem<'static>] =
//     format_description!("[hour]:[minute]:[second].[subsecond]");
//
// #[derive(Debug)]
// pub struct StuckThread<'a> {
//     pub event: Event<'a>,
// }
//
// #[derive(Debug)]
// pub enum Event<'a> {
//     Begin(Begin<'a>, StackTrace<'a>),
//     End(End<'a>),
// }
//
// #[derive(Debug)]
// pub struct Begin<'a> {
//     pub start: time::PrimitiveDateTime,
//     pub thread_id: u32,
//     pub thread_name: &'a str,
//     pub request: &'a str,
//     pub active_duration_ms: i64,
//     pub active_monitor_count: i64,
// }
//
// #[derive(Debug)]
// pub struct End<'a> {
//     pub start: time::PrimitiveDateTime,
//     pub thread_id: u32,
//     pub thread_name: &'a str,
//     pub active_duration_ms: i64,
//     pub active_monitor_count: i64,
// }
//
// pub struct StuckThreadStream<'a>(pub &'a [u8]);
//
// impl<'a> Iterator for StuckThreadStream<'a> {
//     type Item = &'a str;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.0.trim_ascii_start().is_empty() {
//             self.0 = b"";
//             return None;
//         }
//
//         self.0 = self.0.trim_ascii_start();
//
//         let start = 0;
//         let mut offset = 0;
//         let mut iter = self.0.split_inclusive(|b| *b == b'\n').peekable();
//         if let Some(line) = iter.next() {
//             if !line.starts_with(b"[") {
//                 eprintln!("Unreachable");
//                 return None;
//             }
//             offset += line.len();
//         }
//
//         while let Some(line) = iter.next_if(|l| !l.starts_with(b"[")) {
//             offset += line.len();
//         }
//
//         let contents = &self.0[start..start + offset];
//         self.0 = &self.0[start + offset..];
//
//         std::str::from_utf8(contents).ok()
//     }
// }
//
// impl<'a> TryFrom<&'a str> for StuckThread<'a> {
//     type Error = Parse;
//
//     fn try_from(value: &'a str) -> Result<Self, Self::Error> {
//         let mut scanner = Scanner::new(value);
//
//         let meta = scanner.take_until("\n").ok_or(Parse::MetaExtractionError)?;
//         let meta = Event::try_from(meta)?;
//
//         match meta {
//             Event::Begin(begin, _) => {
//                 let stacktrace =
//                     StackTrace::try_from(scanner.remaining()).map_err(Parse::StackParseError)?;
//
//                 Ok(Self {
//                     event: Event::Begin(begin, stacktrace),
//                 })
//             }
//
//             Event::End(end) => Ok(Self {
//                 event: Event::End(end),
//             }),
//         }
//     }
// }
//
// impl<'a> Event<'a> {
//     fn extract_bracket_groups(input: &str) -> Result<Vec<&str>, Parse> {
//         let mut scanner = Scanner::new(input);
//         let mut groups = Vec::with_capacity(8);
//         while let Ok(group) = scanner.take_within("[", "]") {
//             groups.push(group);
//         }
//         Ok(groups)
//     }
//
//     fn parse_thread_id(value: &'a str) -> Result<u32, Parse> {
//         value.parse::<u32>().map_err(|e| Parse::InvalidThreadId {
//             got: value.to_string(),
//             inner: e,
//         })
//     }
//
//     fn parse_comma_separate_i64(value: &'a str) -> Result<i64, Parse> {
//         value
//             .replace(",", "")
//             .parse::<i64>()
//             .map_err(|e| Parse::InvalidActiveDuration {
//                 got: value.to_string(),
//                 inner: e,
//             })
//     }
//
//     fn parse_i64(value: &'a str) -> Result<i64, Parse> {
//         value
//             .parse::<i64>()
//             .map_err(|e| Parse::InvalidActiveThreadCount {
//                 got: value.to_string(),
//                 inner: e,
//             })
//     }
//
//     fn parse_date_time(date: &'a str, time: &'a str) -> Result<PrimitiveDateTime, Parse> {
//         let time = Time::parse(time, TIME_FORMAT).map_err(Parse::InvalidDateTimeFormat)?;
//
//         let date = Date::parse(date, DATE_FORMAT).map_err(Parse::InvalidDateTimeFormat)?;
//
//         let datetime = PrimitiveDateTime::new(date, time);
//         Ok(datetime)
//     }
// }
//
// impl<'a> TryFrom<&'a str> for Event<'a> {
//     type Error = Parse;
//
//     fn try_from(value: &'a str) -> Result<Self, Self::Error> {
//         let mut scanner = Scanner::new(value);
//         let header = scanner.take_until("::").ok_or(Parse::DoubleColonAbsent)?;
//
//         let headers = Event::extract_bracket_groups(header)?;
//         let message = Event::extract_bracket_groups(scanner.remaining())?;
//
//         let [time, date, _, _, _] = headers.as_slice() else {
//             return Err(Parse::IncorrectHeaderInfoCount {
//                 got: headers.iter().map(|s| (*s).to_string()).collect(),
//                 expected: 5,
//             });
//         };
//
//         let start = Event::parse_date_time(date, time)?;
//
//         let mut meta: Event;
//         let message_len = message.len();
//         if message_len == 7 {
//             meta = Event::Begin(Begin::try_from(message)?, Default::default());
//         } else if message_len == 4 || message_len == 3 {
//             meta = Event::End(End::try_from(message)?);
//         } else {
//             return Err(Parse::IncorrectMessageInfoCount {
//                 got: message.iter().map(|s| (*s).to_string()).collect(),
//                 minimum_expected: 3,
//             });
//         }
//
//         match meta {
//             Event::Begin(ref mut b, _) => {
//                 b.start = start
//                     .checked_sub(Duration::milliseconds(b.active_duration_ms))
//                     .ok_or(Parse::DurationOverflow)?;
//             }
//             Event::End(ref mut b) => {
//                 b.start = start
//                     .checked_sub(Duration::milliseconds(b.active_duration_ms))
//                     .ok_or(Parse::DurationOverflow)?;
//             }
//         };
//
//         Ok(meta)
//     }
// }
//
// impl<'a> TryFrom<Vec<&'a str>> for Begin<'a> {
//     type Error = Parse;
//
//     fn try_from(value: Vec<&'a str>) -> Result<Self, Self::Error> {
//         let [
//             thread_name,
//             thread_id,
//             active_duration,
//             _,
//             api_request,
//             _threshold,
//             active_thread_count,
//         ] = value.as_slice()
//         else {
//             return Err(Parse::IncorrectMessageInfoCount {
//                 got: value.iter().map(|s| s.to_string()).collect(),
//                 minimum_expected: 7,
//             });
//         };
//
//         let thread_id = Event::parse_thread_id(thread_id)?;
//         let active_duration = Event::parse_comma_separate_i64(active_duration)?;
//         let active_thread_count = Event::parse_i64(active_thread_count)?;
//
//         Ok(Begin {
//             start: datetime!(1970-01-01 00:00:00.0),
//             active_duration_ms: active_duration,
//             active_monitor_count: active_thread_count,
//             thread_id,
//             thread_name,
//             request: api_request,
//         })
//     }
// }
//
// impl<'a> TryFrom<Vec<&'a str>> for End<'a> {
//     type Error = Parse;
//
//     fn try_from(value: Vec<&'a str>) -> Result<Self, Self::Error> {
//         let mut active_thread_count = None;
//         let thread_name: &str;
//         let thread_id: &str;
//         let active_duration: &str;
//
//         match value.as_slice() {
//             [tn, ti, ad] => {
//                 thread_name = *tn;
//                 thread_id = *ti;
//                 active_duration = *ad;
//             }
//
//             [tn, ti, ad, atc] => {
//                 thread_name = *tn;
//                 thread_id = *ti;
//                 active_duration = *ad;
//                 active_thread_count = Some(atc);
//             }
//
//             _ => {
//                 return Err(Parse::IncorrectMessageInfoCount {
//                     got: value.iter().map(|s| s.to_string()).collect(),
//                     minimum_expected: 3,
//                 });
//             }
//         }
//
//         let thread_id = Event::parse_thread_id(thread_id)?;
//         let active_duration = Event::parse_comma_separate_i64(active_duration)?;
//         let mut active_monitor_count = 0;
//         if let Some(active_thread_count) = active_thread_count {
//             active_monitor_count = Event::parse_i64(active_thread_count)?;
//         }
//
//         Ok(End {
//             start: datetime!(1970-01-01 00:00:00.0),
//             thread_name,
//             thread_id,
//             active_duration_ms: active_duration,
//             active_monitor_count,
//         })
//     }
// }
