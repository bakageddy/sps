//! Tests for `crate::parser::stuckthread`.
//!
//! Sections
//! --------
//!   1.  Helpers / fixtures
//!   2.  `Event::try_from`              — full meta-line parsing
//!         2a. Happy paths (Begin / End-4 / End-3)
//!         2b. Header-group errors
//!         2c. Message-group errors
//!         2d. Numeric parsing errors
//!         2e. Date/time parsing errors
//!         2f. Duration overflow
//!         2g. Start-time arithmetic
//!   3.  `Begin::try_from`              — slice-of-bytes constructor
//!   4.  `End::try_from`                — slice-of-bytes constructor (3/4-form)
//!   5.  `StuckThread::try_from`        — meta + optional stack trace
//!   6.  Zero-copy / lifetime sanity
//!
//! Notes
//! -----
//! * All inputs are `&[u8]` — the parser no longer accepts `&str`.
//! * `start` is a unix-millis `u64` (UTC) computed as
//!   `header_unix_ms - active_duration_ms`. Tests assert the relation rather
//!   than hard-coded values, so they remain valid if the fixture changes.
//! * The header timestamp format is `HH:MM:SS.sss` + `DD-MM-YYYY` (assumed UTC).

use crate::error::stuckthread::Parse;
use crate::parser::stacktrace::Element;
use crate::parser::stuckthread::{Begin, End, Event, StuckThread};
use crate::util::ToUnixMillis;

use time::macros::{datetime, format_description};
use time::{Date, PrimitiveDateTime, Time};

// ============================================================================
// 1. Helpers / fixtures
// ============================================================================

/// Canonical Begin meta line (single line, trailing '\n').
const BEGIN_META: &[u8] = b"[15:52:17.284]|[16-02-2026]|[org.apache.catalina.valves.StuckThreadDetectionValve]|[WARN]|[90]| :: Thread [/api/v3/requests/3472132/request_detail-1771237317720_###_] (id=[226300]) has been active for [19,273] milliseconds (since [2/16/26 3:51 PM]) to serve the same request for [http://sdp-loadt-3:8080/api/v3/requests/3472132/request_detail?includes=%5B%22_links%22%5D&_=1687928484586] and may be stuck (configured threshold for this StuckThreadDetectionValve is [10] seconds). There is/are [79] thread(s) in total that are monitored by this Valve and may be stuck.\n";

/// End meta line with the 4-group message form (includes monitor count).
const END_META_4: &[u8] = b"[15:52:17.351]|[16-02-2026]|[org.apache.catalina.valves.StuckThreadDetectionValve]|[WARN]|[90]| :: Thread [] (id=[226293]) was previously reported to be stuck but has completed. It was active for approximately [27,650] milliseconds. There is/are still [78] thread(s) that are monitored by this Valve and may be stuck.\n";

/// End meta line with the 3-group message form (no monitor count).
const END_META_3: &[u8] = b"[15:52:17.351]|[16-02-2026]|[org.apache.catalina.valves.StuckThreadDetectionValve]|[WARN]|[90]| :: Thread [worker-7] (id=[42]) finished after [1,500] milliseconds.\n";

/// A small but valid stack trace (Throwable + two `at` frames).
const SMALL_STACKTRACE: &[u8] = b"java.lang.Throwable\n\tat java.base@17.0.17/jdk.internal.misc.Unsafe.park(Native Method)\n\tat com.zoho.cp.ObjectStack.acquirePermitAndPollLastElement(ObjectStack.java:23)\n";

/// Build a full Begin record (meta + stack trace).
fn begin_record() -> Vec<u8> {
    let mut v = Vec::with_capacity(BEGIN_META.len() + SMALL_STACKTRACE.len());
    v.extend_from_slice(BEGIN_META);
    v.extend_from_slice(SMALL_STACKTRACE);
    v
}

/// Compute the header timestamp's unix-ms using the same convention as the
/// parser (UTC). Lets tests assert the start-time relation without hard-coding
/// magic numbers.
fn unix_ms(h: u8, m: u8, s: u8, ms: u16, day: u8, month: time::Month, year: i32) -> u64 {
    let time = Time::from_hms_milli(h, m, s, ms).unwrap();
    let date = Date::from_calendar_date(year, month, day).unwrap();
    PrimitiveDateTime::new(date, time).to_unix_millis().unwrap()
}

// ============================================================================
// 2. Event::try_from
// ============================================================================

// ---- 2a. Happy paths --------------------------------------------------------

#[test]
fn event_begin_parses() {
    let ev = Event::try_from(BEGIN_META).expect("BEGIN_META must parse");
    assert!(matches!(ev, Event::Begin(_, _)));
}

#[test]
fn event_begin_field_values() {
    let Event::Begin(b, _) = Event::try_from(BEGIN_META).unwrap() else {
        panic!("expected Begin");
    };
    assert_eq!(b.tid, 226300);
    assert_eq!(b.active_duration_ms, 19_273);
    assert_eq!(b.active_monitor_count, 79);
    assert_eq!(
        b.name,
        b"/api/v3/requests/3472132/request_detail-1771237317720_###_".as_slice()
    );
    assert!(b
        .request
        .starts_with(b"http://sdp-loadt-3:8080/api/v3/requests/3472132/request_detail"));
}

#[test]
fn event_begin_trace_is_default_empty_at_event_layer() {
    // Event::try_from for Begin does NOT parse the stack trace
    // (that's stitched in by StuckThread::try_from). The trace slot is Default.
    let Event::Begin(_, trace) = Event::try_from(BEGIN_META).unwrap() else {
        panic!("expected Begin");
    };
    assert!(
        trace.0.is_empty(),
        "Event-level Begin should carry an empty default Trace"
    );
}

#[test]
fn event_end_4_groups_parses() {
    let ev = Event::try_from(END_META_4).expect("END_META_4 must parse");
    assert!(matches!(ev, Event::End(_)));
}

#[test]
fn event_end_4_group_field_values() {
    let Event::End(e) = Event::try_from(END_META_4).unwrap() else {
        panic!("expected End");
    };
    assert_eq!(e.tid, 226293);
    assert_eq!(e.active_duration_ms, 27_650);
    assert_eq!(e.active_monitor_count, 78);
    assert_eq!(e.name, b"".as_slice());
}

#[test]
fn event_end_3_groups_parses_with_zero_monitor_count() {
    let Event::End(e) = Event::try_from(END_META_3).unwrap() else {
        panic!("expected End");
    };
    assert_eq!(e.tid, 42);
    assert_eq!(e.active_duration_ms, 1_500);
    // 3-group form omits the monitor count → defaults to 0.
    assert_eq!(e.active_monitor_count, 0);
    assert_eq!(e.name, b"worker-7".as_slice());
}

// ---- 2b. Header-group errors -----------------------------------------------

#[test]
fn event_missing_double_colon() {
    let input = b"[15:52:17.284]|[16-02-2026]|[c]|[WARN]|[90]| no double colon here";
    let r = Event::try_from(input.as_slice());
    assert!(matches!(r, Err(Parse::DoubleColonAbsent)), "got {r:?}");
}

#[test]
fn event_header_too_few_groups() {
    // 4 header groups instead of required 5
    let input = b"[15:52:17.284]|[16-02-2026]|[class]|[WARN]| :: Thread [n] (id=[1]) was previously reported to be stuck but has completed. It was active for approximately [100] milliseconds.";
    let r = Event::try_from(input.as_slice());
    assert!(
        matches!(r, Err(Parse::IncorrectHeaderInfoCount { expected: 5, .. })),
        "got {r:?}"
    );
}

#[test]
fn event_header_too_many_groups() {
    // 6 header groups instead of required 5
    let input = b"[15:52:17.284]|[16-02-2026]|[class]|[WARN]|[90]|[extra]| :: Thread [n] (id=[1]) finished after [100] milliseconds.";
    let r = Event::try_from(input.as_slice());
    assert!(
        matches!(r, Err(Parse::IncorrectHeaderInfoCount { expected: 5, .. })),
        "got {r:?}"
    );
}

#[test]
fn event_empty_input() {
    let r = Event::try_from(b"".as_slice());
    // Empty input has no `::` → DoubleColonAbsent.
    assert!(matches!(r, Err(Parse::DoubleColonAbsent)), "got {r:?}");
}

// ---- 2c. Message-group errors ----------------------------------------------

#[test]
fn event_message_too_few_groups() {
    // 2 message groups — matches neither Begin (7) nor End (3/4).
    let input = b"[15:52:17.284]|[16-02-2026]|[c]|[WARN]|[90]| :: Thread [n] only [2] groups here.";
    let r = Event::try_from(input.as_slice());
    assert!(
        matches!(
            r,
            Err(Parse::IncorrectMessageInfoCount { minimum_expected: 3, .. })
        ),
        "got {r:?}"
    );
}

#[test]
fn event_message_five_groups_unrecognized() {
    // 5 message groups — not 3, 4, or 7.
    let input = b"[15:52:17.284]|[16-02-2026]|[c]|[WARN]|[90]| :: [a] [b] [c] [d] [e]";
    let r = Event::try_from(input.as_slice());
    assert!(
        matches!(
            r,
            Err(Parse::IncorrectMessageInfoCount { minimum_expected: 3, .. })
        ),
        "got {r:?}"
    );
}

#[test]
fn event_message_six_groups_unrecognized() {
    let input = b"[15:52:17.284]|[16-02-2026]|[c]|[WARN]|[90]| :: [a] [b] [c] [d] [e] [f]";
    let r = Event::try_from(input.as_slice());
    assert!(
        matches!(
            r,
            Err(Parse::IncorrectMessageInfoCount { minimum_expected: 3, .. })
        ),
        "got {r:?}"
    );
}

// ---- 2d. Numeric parsing errors --------------------------------------------

#[test]
fn event_invalid_thread_id() {
    let input = b"[15:52:17.351]|[16-02-2026]|[c]|[WARN]|[90]| :: Thread [] (id=[abc]) was previously reported to be stuck but has completed. It was active for approximately [100] milliseconds.";
    let r = Event::try_from(input.as_slice());
    assert!(matches!(r, Err(Parse::InvalidThreadId { .. })), "got {r:?}");
}

#[test]
fn event_invalid_duration() {
    let input = b"[15:52:17.351]|[16-02-2026]|[c]|[WARN]|[90]| :: Thread [n] (id=[1]) was previously reported to be stuck but has completed. It was active for approximately [not_a_number] milliseconds.";
    let r = Event::try_from(input.as_slice());
    assert!(
        matches!(r, Err(Parse::InvalidActiveDuration { .. })),
        "got {r:?}"
    );
}

#[test]
fn event_invalid_monitor_count_end() {
    let input = b"[15:52:17.351]|[16-02-2026]|[c]|[WARN]|[90]| :: Thread [] (id=[1]) was previously reported to be stuck but has completed. It was active for approximately [100] milliseconds. There is/are still [zzz] thread(s).";
    let r = Event::try_from(input.as_slice());
    assert!(
        matches!(r, Err(Parse::InvalidActiveThreadCount { .. })),
        "got {r:?}"
    );
}

#[test]
fn event_begin_invalid_monitor_count() {
    // BEGIN's count is the 7th group ([79] in the canonical fixture). Replace it with garbage.
    let input = b"[15:52:17.284]|[16-02-2026]|[c]|[WARN]|[90]| :: Thread [n] (id=[1]) has been active for [100] milliseconds (since [d]) to serve the same request for [u] and may be stuck (configured threshold for this StuckThreadDetectionValve is [10] seconds). There is/are [oops] thread(s) total.";
    let r = Event::try_from(input.as_slice());
    assert!(
        matches!(r, Err(Parse::InvalidActiveThreadCount { .. })),
        "got {r:?}"
    );
}

// ---- 2e. Date/time parsing errors ------------------------------------------

#[test]
fn event_invalid_time_format() {
    let input = b"[notatime]|[16-02-2026]|[c]|[WARN]|[90]| :: Thread [] (id=[1]) was previously reported to be stuck but has completed. It was active for approximately [10] milliseconds.";
    let r = Event::try_from(input.as_slice());
    assert!(
        matches!(r, Err(Parse::InvalidDateTimeFormat(_))),
        "got {r:?}"
    );
}

#[test]
fn event_invalid_date_format() {
    let input = b"[15:52:17.351]|[notadate]|[c]|[WARN]|[90]| :: Thread [] (id=[1]) was previously reported to be stuck but has completed. It was active for approximately [10] milliseconds.";
    let r = Event::try_from(input.as_slice());
    assert!(
        matches!(r, Err(Parse::InvalidDateTimeFormat(_))),
        "got {r:?}"
    );
}

// ---- 2f. Duration overflow --------------------------------------------------

#[test]
fn event_duration_overflow() {
    // Duration u32::MAX ms > header millis since epoch? No — header is in 2026.
    // But duration well above the header millis is impossible to construct
    // with u32 (~49 days). Instead, force a near-epoch header date with a
    // large duration to drive subtraction underflow.
    // Header date 01-01-1970, time 00:00:00.000, duration 100ms → 0 - 100 underflows.
    let input = b"[00:00:00.000]|[01-01-1970]|[c]|[WARN]|[90]| :: Thread [] (id=[1]) was previously reported to be stuck but has completed. It was active for approximately [100] milliseconds.";
    let r = Event::try_from(input.as_slice());
    assert!(matches!(r, Err(Parse::DurationOverflow)), "got {r:?}");
}

#[test]
fn event_duration_exactly_header_is_ok() {
    // Header == duration → start = 0. Should NOT overflow.
    let input = b"[00:00:00.100]|[01-01-1970]|[c]|[WARN]|[90]| :: Thread [] (id=[1]) was previously reported to be stuck but has completed. It was active for approximately [100] milliseconds.";
    let ev = Event::try_from(input.as_slice()).expect("exact-equality must not underflow");
    match ev {
        Event::End(e) => assert_eq!(e.start, 0),
        _ => panic!("expected End"),
    }
}

// ---- 2g. Start-time arithmetic ---------------------------------------------

#[test]
fn event_begin_start_equals_header_minus_duration() {
    let header = unix_ms(15, 52, 17, 284, 16, time::Month::February, 2026);
    let Event::Begin(b, _) = Event::try_from(BEGIN_META).unwrap() else {
        panic!("expected Begin");
    };
    assert_eq!(b.start, header - 19_273);
}

#[test]
fn event_end_4_start_equals_header_minus_duration() {
    let header = unix_ms(15, 52, 17, 351, 16, time::Month::February, 2026);
    let Event::End(e) = Event::try_from(END_META_4).unwrap() else {
        panic!("expected End");
    };
    assert_eq!(e.start, header - 27_650);
}

#[test]
fn event_end_3_start_equals_header_minus_duration() {
    let header = unix_ms(15, 52, 17, 351, 16, time::Month::February, 2026);
    let Event::End(e) = Event::try_from(END_META_3).unwrap() else {
        panic!("expected End");
    };
    assert_eq!(e.start, header - 1_500);
}

// ============================================================================
// 3. Begin::try_from(&[&[u8]])
// ============================================================================

#[test]
fn begin_from_slice_happy_path() {
    let groups: [&[u8]; 7] = [
        b"my-thread-1",
        b"42",
        b"5,000",
        b"2/16/26 3:51 PM", // ignored
        b"http://example.com/api",
        b"10", // threshold (ignored)
        b"3",
    ];
    let b = Begin::try_from(&groups[..]).expect("must parse");
    assert_eq!(b.tid, 42);
    assert_eq!(b.active_duration_ms, 5_000);
    assert_eq!(b.active_monitor_count, 3);
    assert_eq!(b.name, b"my-thread-1".as_slice());
    assert_eq!(b.request, b"http://example.com/api".as_slice());
    // start is not computed at this layer; Event::try_from fills it.
    assert_eq!(b.start, 0);
}

#[test]
fn begin_from_slice_large_duration_with_commas() {
    let groups: [&[u8]; 7] = [
        b"t",
        b"1",
        b"1,234,567",
        b"date",
        b"http://example.com",
        b"10",
        b"100",
    ];
    let b = Begin::try_from(&groups[..]).unwrap();
    assert_eq!(b.active_duration_ms, 1_234_567);
}

#[test]
fn begin_from_slice_duration_without_commas() {
    let groups: [&[u8]; 7] = [
        b"t",
        b"7",
        b"500",
        b"d",
        b"http://example.com",
        b"10",
        b"1",
    ];
    let b = Begin::try_from(&groups[..]).unwrap();
    assert_eq!(b.active_duration_ms, 500);
}

#[test]
fn begin_from_slice_large_tid() {
    let groups: [&[u8]; 7] = [
        b"t",
        b"18446744073709551615", // u64::MAX
        b"100",
        b"d",
        b"u",
        b"10",
        b"1",
    ];
    let b = Begin::try_from(&groups[..]).unwrap();
    assert_eq!(b.tid, u64::MAX);
}

#[test]
fn begin_from_slice_wrong_count_three() {
    let groups: [&[u8]; 3] = [b"only", b"three", b"items"];
    let r = Begin::try_from(&groups[..]);
    assert!(
        matches!(r, Err(Parse::IncorrectMessageInfoCount { .. })),
        "got {r:?}"
    );
}

#[test]
fn begin_from_slice_wrong_count_eight() {
    let groups: [&[u8]; 8] = [b"a", b"1", b"100", b"d", b"u", b"10", b"5", b"extra"];
    let r = Begin::try_from(&groups[..]);
    assert!(
        matches!(r, Err(Parse::IncorrectMessageInfoCount { .. })),
        "got {r:?}"
    );
}

#[test]
fn begin_from_slice_invalid_tid() {
    let groups: [&[u8]; 7] = [b"t", b"NaN", b"100", b"d", b"u", b"10", b"1"];
    let r = Begin::try_from(&groups[..]);
    assert!(matches!(r, Err(Parse::InvalidThreadId { .. })), "got {r:?}");
}

#[test]
fn begin_from_slice_invalid_duration() {
    let groups: [&[u8]; 7] = [b"t", b"1", b"abc", b"d", b"u", b"10", b"1"];
    let r = Begin::try_from(&groups[..]);
    assert!(
        matches!(r, Err(Parse::InvalidActiveDuration { .. })),
        "got {r:?}"
    );
}

#[test]
fn begin_from_slice_invalid_monitor_count() {
    let groups: [&[u8]; 7] = [b"t", b"1", b"100", b"d", b"u", b"10", b"xyz"];
    let r = Begin::try_from(&groups[..]);
    assert!(
        matches!(r, Err(Parse::InvalidActiveThreadCount { .. })),
        "got {r:?}"
    );
}

// ============================================================================
// 4. End::try_from(&[&[u8]])
// ============================================================================

#[test]
fn end_from_slice_3_groups() {
    let groups: [&[u8]; 3] = [b"", b"226293", b"27,650"];
    let e = End::try_from(&groups[..]).unwrap();
    assert_eq!(e.tid, 226_293);
    assert_eq!(e.active_duration_ms, 27_650);
    assert_eq!(e.active_monitor_count, 0); // defaulted
    assert_eq!(e.name, b"".as_slice());
}

#[test]
fn end_from_slice_4_groups() {
    let groups: [&[u8]; 4] = [b"worker-1", b"7", b"1,500", b"42"];
    let e = End::try_from(&groups[..]).unwrap();
    assert_eq!(e.tid, 7);
    assert_eq!(e.active_duration_ms, 1_500);
    assert_eq!(e.active_monitor_count, 42);
    assert_eq!(e.name, b"worker-1".as_slice());
}

#[test]
fn end_from_slice_wrong_count_two() {
    let groups: [&[u8]; 2] = [b"name", b"123"];
    let r = End::try_from(&groups[..]);
    assert!(
        matches!(r, Err(Parse::IncorrectMessageInfoCount { .. })),
        "got {r:?}"
    );
}

#[test]
fn end_from_slice_wrong_count_five() {
    let groups: [&[u8]; 5] = [b"a", b"1", b"100", b"5", b"extra"];
    let r = End::try_from(&groups[..]);
    assert!(
        matches!(r, Err(Parse::IncorrectMessageInfoCount { .. })),
        "got {r:?}"
    );
}

#[test]
fn end_from_slice_invalid_tid() {
    let groups: [&[u8]; 4] = [b"n", b"NaN", b"100", b"1"];
    let r = End::try_from(&groups[..]);
    assert!(matches!(r, Err(Parse::InvalidThreadId { .. })), "got {r:?}");
}

#[test]
fn end_from_slice_invalid_duration() {
    let groups: [&[u8]; 4] = [b"n", b"1", b"abc", b"1"];
    let r = End::try_from(&groups[..]);
    assert!(
        matches!(r, Err(Parse::InvalidActiveDuration { .. })),
        "got {r:?}"
    );
}

#[test]
fn end_from_slice_invalid_monitor_count() {
    let groups: [&[u8]; 4] = [b"n", b"1", b"100", b"xyz"];
    let r = End::try_from(&groups[..]);
    assert!(
        matches!(r, Err(Parse::InvalidActiveThreadCount { .. })),
        "got {r:?}"
    );
}

#[test]
fn end_from_slice_3_groups_ignores_monitor_field_entirely() {
    // 3-group form has no monitor count slot to fail on.
    let groups: [&[u8]; 3] = [b"n", b"1", b"100"];
    let e = End::try_from(&groups[..]).unwrap();
    assert_eq!(e.active_monitor_count, 0);
}

// ============================================================================
// 5. StuckThread::try_from
// ============================================================================

#[test]
fn stuckthread_begin_with_stacktrace() {
    let buf = begin_record();
    let st = StuckThread::try_from(buf.as_slice()).expect("must parse");
    let StuckThread(Event::Begin(b, trace)) = st else {
        panic!("expected Begin variant");
    };
    assert_eq!(b.tid, 226300);
    assert_eq!(b.active_duration_ms, 19_273);
    assert!(
        !trace.0.is_empty(),
        "Begin record must carry a parsed stack trace"
    );
    // First non-lock element should be a frame.
    let frame_count = trace
        .0
        .iter()
        .filter(|e| matches!(e, Element::Elem { .. }))
        .count();
    assert!(frame_count >= 1, "expected >= 1 frame, got {trace:?}");
}

#[test]
fn stuckthread_end_record_single_line() {
    // End records are single-line (no stack trace follows). The parser splits
    // on '\n' for the meta line; the rest may be empty and that's fine.
    let st = StuckThread::try_from(END_META_4).expect("must parse");
    assert!(matches!(st.0, Event::End(_)));
}

#[test]
fn stuckthread_end_3group_form() {
    let st = StuckThread::try_from(END_META_3).expect("must parse");
    let StuckThread(Event::End(e)) = st else {
        panic!("expected End");
    };
    assert_eq!(e.tid, 42);
    assert_eq!(e.active_monitor_count, 0);
}

#[test]
fn stuckthread_no_newline_is_meta_extraction_error() {
    let r = StuckThread::try_from(b"[no][newline][here]".as_slice());
    assert!(matches!(r, Err(Parse::MetaExtractionError)), "got {r:?}");
}

#[test]
fn stuckthread_begin_meta_without_stacktrace_errors() {
    // Begin meta but no `java.lang.Throwable` body → stack-trace parser fails.
    let r = StuckThread::try_from(BEGIN_META);
    assert!(
        matches!(r, Err(Parse::StackParseError(_))),
        "got {r:?} — Begin with no stack-trace body must surface StackParseError"
    );
}

#[test]
fn stuckthread_propagates_event_errors() {
    // Missing `::` is an Event-layer error, surfaced through StuckThread.
    let r = StuckThread::try_from(b"no double colon\n".as_slice());
    assert!(matches!(r, Err(Parse::DoubleColonAbsent)), "got {r:?}");
}

#[test]
fn stuckthread_empty_input_is_meta_extraction_error() {
    let r = StuckThread::try_from(b"".as_slice());
    assert!(matches!(r, Err(Parse::MetaExtractionError)), "got {r:?}");
}

#[test]
fn stuckthread_begin_start_relation_holds() {
    let buf = begin_record();
    let st = StuckThread::try_from(buf.as_slice()).unwrap();
    let StuckThread(Event::Begin(b, _)) = st else {
        panic!("expected Begin");
    };
    let header = unix_ms(15, 52, 17, 284, 16, time::Month::February, 2026);
    assert_eq!(b.start, header - b.active_duration_ms as u64);
}

// ============================================================================
// 6. Zero-copy / lifetime sanity
// ============================================================================

#[test]
fn begin_fields_borrow_from_input_buffer() {
    // The returned `name` / `request` slices must point INTO the input buffer,
    // not into a separate allocation. We assert this by comparing pointer ranges.
    let buf = begin_record();
    let base = buf.as_ptr() as usize;
    let end = base + buf.len();

    let st = StuckThread::try_from(buf.as_slice()).unwrap();
    let StuckThread(Event::Begin(b, _)) = st else {
        panic!("expected Begin");
    };

    let name_ptr = b.name.as_ptr() as usize;
    let req_ptr = b.request.as_ptr() as usize;
    assert!(
        (base..end).contains(&name_ptr),
        "name slice escaped input buffer"
    );
    assert!(
        (base..end).contains(&req_ptr),
        "request slice escaped input buffer"
    );
}

#[test]
fn end_fields_borrow_from_input_buffer() {
    let buf: Vec<u8> = END_META_4.to_vec();
    let base = buf.as_ptr() as usize;
    let end_addr = base + buf.len();

    let StuckThread(Event::End(e)) = StuckThread::try_from(buf.as_slice()).unwrap() else {
        panic!("expected End");
    };

    let name_ptr = e.name.as_ptr() as usize;
    // Empty slices have an implementation-defined pointer; only check non-empty fields.
    if !e.name.is_empty() {
        assert!((base..end_addr).contains(&name_ptr));
    }
}

// ============================================================================
// 7. Sanity: PrimitiveDateTime macro fixture (regression for ToUnixMillis)
// ============================================================================

#[test]
fn unix_ms_helper_matches_datetime_macro() {
    // Belt-and-braces: confirm the helper agrees with a compile-time literal.
    let dt = datetime!(2026-02-16 15:52:17.284);
    assert_eq!(
        unix_ms(15, 52, 17, 284, 16, time::Month::February, 2026),
        dt.to_unix_millis().unwrap()
    );
}

#[test]
fn date_format_round_trip_smoke() {
    // The parser uses "DD-MM-YYYY"; ensure our understanding matches.
    let fmt = format_description!("[day]-[month]-[year]");
    let d = Date::parse("16-02-2026", fmt).unwrap();
    assert_eq!(d.year(), 2026);
    assert_eq!(d.month(), time::Month::February);
    assert_eq!(d.day(), 16);
}
