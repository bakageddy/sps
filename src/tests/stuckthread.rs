use crate::error::stuckthread::Parse;
use crate::stuckthread::{
    StuckThread, StuckThreadMeta, StuckThreadMetaBegin, StuckThreadMetaEnd, StuckThreadStream,
};

// ============================================================
// Helper: Build meta lines
// ============================================================

#[allow(unused)]
const BEGIN_META: &str = "[15:52:17.284]|[16-02-2026]|[org.apache.catalina.valves.StuckThreadDetectionValve]|[WARN]|[90]| :: Thread [/api/v3/requests/3472132/request_detail-1771237317720_###_] (id=[226300]) has been active for [19,273] milliseconds (since [2/16/26 3:51 PM]) to serve the same request for [http://sdp-loadt-3:8080/api/v3/requests/3472132/request_detail?includes=%5B%22_links%22%5D&_=1687928484586] and may be stuck (configured threshold for this StuckThreadDetectionValve is [10] seconds). There is/are [79] thread(s) in total that are monitored by this Valve and may be stuck.\n";

const END_META_4_GROUPS: &str = "[15:52:17.351]|[16-02-2026]|[org.apache.catalina.valves.StuckThreadDetectionValve]|[WARN]|[90]| :: Thread [] (id=[226293]) was previously reported to be stuck but has completed. It was active for approximately [27,650] milliseconds. There is/are still [78] thread(s) that are monitored by this Valve and may be stuck.\n";

const END_META_4_GROUPS_B: &str = "[15:52:17.651]|[16-02-2026]|[org.apache.catalina.valves.StuckThreadDetectionValve]|[WARN]|[90]| :: Thread [] (id=[226802]) was previously reported to be stuck but has completed. It was active for approximately [38,600] milliseconds. There is/are still [77] thread(s) that are monitored by this Valve and may be stuck.\n";

const SMALL_STACKTRACE: &str = "java.lang.Throwable\n\tat java.base@17.0.17/jdk.internal.misc.Unsafe.park(Native Method)\n\tat com.zoho.cp.ObjectStack.acquirePermitAndPollLastElement(ObjectStack.java:23)\n";

// ============================================================
// StuckThreadMeta — bracket group extraction & meta parsing
// ============================================================

#[test]
fn meta_begin_parses_correctly() {
    let result = StuckThreadMeta::try_from(BEGIN_META);
    assert!(result.is_ok(), "failed: {result:?}");
    let meta = result.unwrap();
    assert!(matches!(meta, StuckThreadMeta::Begin(_)));
}

#[test]
fn meta_begin_fields() {
    let meta = StuckThreadMeta::try_from(BEGIN_META).unwrap();
    match meta {
        StuckThreadMeta::Begin(b) => {
            assert_eq!(
                b.thread_name,
                "/api/v3/requests/3472132/request_detail-1771237317720_###_"
            );
            assert_eq!(b.thread_id, 226300);
            assert_eq!(b.active_duration_ms, 19273);
            assert_eq!(b.active_monitor_count, 79);
            assert!(
                b.request
                    .starts_with("http://sdp-loadt-3:8080/api/v3/requests/3472132/request_detail")
            );
        }
        _ => panic!("expected Begin"),
    }
}

#[test]
fn meta_begin_start_time_is_adjusted() {
    // start = header_timestamp - active_duration_ms
    // header: 15:52:17.284 on 16-02-2026
    // duration: 19,273 ms
    // start should be ~15:51:58.011
    let meta = StuckThreadMeta::try_from(BEGIN_META).unwrap();
    match meta {
        StuckThreadMeta::Begin(b) => {
            // The start time should be before the header timestamp
            let start_second = b.start.second();
            let start_minute = b.start.minute();
            // 15:52:17.284 - 19.273s ≈ 15:51:58.011
            assert_eq!(start_minute, 51);
            assert_eq!(start_second, 58);
        }
        _ => panic!("expected Begin"),
    }
}

#[test]
fn meta_end_4_groups() {
    let result = StuckThreadMeta::try_from(END_META_4_GROUPS);
    assert!(result.is_ok(), "failed: {result:?}");
    let meta = result.unwrap();
    assert!(matches!(meta, StuckThreadMeta::End(_)));
}

#[test]
fn meta_end_fields() {
    let meta = StuckThreadMeta::try_from(END_META_4_GROUPS).unwrap();
    match meta {
        StuckThreadMeta::End(e) => {
            assert_eq!(e.thread_name, "");
            assert_eq!(e.thread_id, 226293);
            assert_eq!(e.active_duration_ms, 27650);
            assert_eq!(e.active_monitor_count, 78);
        }
        _ => panic!("expected End"),
    }
}

#[test]
fn meta_end_second_record() {
    let meta = StuckThreadMeta::try_from(END_META_4_GROUPS_B).unwrap();
    match meta {
        StuckThreadMeta::End(e) => {
            assert_eq!(e.thread_id, 226802);
            assert_eq!(e.active_duration_ms, 38600);
            assert_eq!(e.active_monitor_count, 77);
        }
        _ => panic!("expected End"),
    }
}

#[test]
fn meta_end_start_time_is_adjusted() {
    let meta = StuckThreadMeta::try_from(END_META_4_GROUPS).unwrap();
    match meta {
        StuckThreadMeta::End(e) => {
            // 15:52:17.351 - 27,650ms ≈ 15:51:49.701
            assert_eq!(e.start.minute(), 51);
            assert_eq!(e.start.second(), 49);
        }
        _ => panic!("expected End"),
    }
}

// ============================================================
// StuckThreadMeta — error cases
// ============================================================

#[test]
fn meta_missing_double_colon() {
    let input = "[15:52:17.284]|[16-02-2026]|[class]|[WARN]|[90]| no double colon here";
    let result = StuckThreadMeta::try_from(input);
    assert!(result.is_err());
    assert!(matches!(result, Err(Parse::DoubleColonAbsent)));
}

#[test]
fn meta_wrong_header_group_count() {
    // Only 3 header groups instead of 5
    let input = "[15:52:17.284]|[16-02-2026]|[class]| :: Thread [name] (id=[1]) has been active for [100] milliseconds (since [now]) to serve the same request for [url] and may be stuck (configured threshold for this StuckThreadDetectionValve is [10] seconds). There is/are [1] thread(s) in total that are monitored by this Valve and may be stuck.";
    let result = StuckThreadMeta::try_from(input);
    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(Parse::IncorrectHeaderInfoCount { .. })
    ));
}

#[test]
fn meta_wrong_message_group_count() {
    // Only 2 message groups — not enough for Begin (7) or End (3-4)
    let input =
        "[15:52:17.284]|[16-02-2026]|[class]|[WARN]|[90]| :: Thread [name] only [2] groups.";
    let result = StuckThreadMeta::try_from(input);
    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(Parse::IncorrectMessageInfoCount { .. })
    ));
}

#[test]
fn meta_invalid_thread_id() {
    // thread_id is "abc" instead of a number
    let input = "[15:52:17.284]|[16-02-2026]|[class]|[WARN]|[90]| :: Thread [] (id=[abc]) was previously reported to be stuck but has completed. It was active for approximately [100] milliseconds.";
    let result = StuckThreadMeta::try_from(input);
    assert!(result.is_err());
}

#[test]
fn meta_invalid_duration() {
    // Duration is not a number
    let input = "[15:52:17.284]|[16-02-2026]|[class]|[WARN]|[90]| :: Thread [] (id=[123]) was previously reported to be stuck but has completed. It was active for approximately [not_a_number] milliseconds.";
    let result = StuckThreadMeta::try_from(input);
    assert!(result.is_err());
}

#[test]
fn meta_empty_string() {
    let result = StuckThreadMeta::try_from("");
    assert!(result.is_err());
}

// ============================================================
// StuckThreadMetaBegin — from bracket groups directly
// ============================================================

#[test]
fn meta_begin_from_groups() {
    let groups = vec![
        "my-thread-1",
        "42",
        "5,000",
        "2/16/26 3:51 PM",
        "http://example.com/api",
        "10",
        "3",
    ];
    let result = StuckThreadMetaBegin::try_from(groups);
    assert!(result.is_ok(), "failed: {result:?}");
    let b = result.unwrap();
    assert_eq!(b.thread_name, "my-thread-1");
    assert_eq!(b.thread_id, 42);
    assert_eq!(b.active_duration_ms, 5000);
    assert_eq!(b.request, "http://example.com/api");
    assert_eq!(b.active_monitor_count, 3);
}

#[test]
fn meta_begin_from_groups_wrong_count() {
    let groups = vec!["only", "three", "items"];
    let result = StuckThreadMetaBegin::try_from(groups);
    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(Parse::IncorrectMessageInfoCount { .. })
    ));
}

#[test]
fn meta_begin_comma_in_duration() {
    let groups = vec![
        "thread",
        "1",
        "21,982",
        "since",
        "http://example.com",
        "10",
        "60",
    ];
    let result = StuckThreadMetaBegin::try_from(groups).unwrap();
    assert_eq!(result.active_duration_ms, 21982);
}

// ============================================================
// StuckThreadMetaEnd — from bracket groups directly
// ============================================================

#[test]
fn meta_end_from_3_groups() {
    let groups = vec!["", "226293", "27,650"];
    let result = StuckThreadMetaEnd::try_from(groups);
    assert!(result.is_ok(), "failed: {result:?}");
    let e = result.unwrap();
    assert_eq!(e.thread_name, "");
    assert_eq!(e.thread_id, 226293);
    assert_eq!(e.active_duration_ms, 27650);
    assert_eq!(e.active_monitor_count, 0); // no count provided → default 0
}

#[test]
fn meta_end_from_4_groups() {
    let groups = vec!["", "226293", "27,650", "78"];
    let result = StuckThreadMetaEnd::try_from(groups);
    assert!(result.is_ok(), "failed: {result:?}");
    let e = result.unwrap();
    assert_eq!(e.thread_id, 226293);
    assert_eq!(e.active_duration_ms, 27650);
    assert_eq!(e.active_monitor_count, 78);
}

#[test]
fn meta_end_from_groups_wrong_count() {
    let groups = vec!["too", "many", "items", "here", "extra"];
    let result = StuckThreadMetaEnd::try_from(groups);
    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(Parse::IncorrectMessageInfoCount { .. })
    ));
}

// ============================================================
// StuckThread — full record (meta + optional stack trace)
// ============================================================

#[test]
fn stuckthread_begin_with_stacktrace() {
    let input = format!("{}{}", BEGIN_META, SMALL_STACKTRACE);
    let result = StuckThread::try_from(input.as_str());
    assert!(result.is_ok(), "failed: {result:?}");
    let st = result.unwrap();
    assert!(matches!(st.meta, StuckThreadMeta::Begin(_)));
    assert!(st.st.is_some());
    assert!(st.st.unwrap().traces.len() >= 1);
}

#[test]
fn stuckthread_begin_meta_fields_preserved() {
    let input = format!("{}{}", BEGIN_META, SMALL_STACKTRACE);
    let st = StuckThread::try_from(input.as_str()).unwrap();
    match st.meta {
        StuckThreadMeta::Begin(ref b) => {
            assert_eq!(b.thread_id, 226300);
            assert_eq!(b.active_duration_ms, 19273);
        }
        _ => panic!("expected Begin"),
    }
}

#[test]
fn stuckthread_end_no_stacktrace() {
    // End records are single-line, no stack trace
    let input = END_META_4_GROUPS;
    let result = StuckThread::try_from(input);
    assert!(result.is_ok(), "failed: {result:?}");
    let st = result.unwrap();
    assert!(matches!(st.meta, StuckThreadMeta::End(_)));
    assert!(st.st.is_none());
}

#[test]
fn stuckthread_end_fields_preserved() {
    let st = StuckThread::try_from(END_META_4_GROUPS).unwrap();
    match st.meta {
        StuckThreadMeta::End(ref e) => {
            assert_eq!(e.thread_id, 226293);
            assert_eq!(e.active_duration_ms, 27650);
            assert_eq!(e.active_monitor_count, 78);
        }
        _ => panic!("expected End"),
    }
}

// ============================================================
// StuckThreadStream — multi-record parsing
// ============================================================

#[test]
fn stream_parse_single_end_record() {
    let input = format!("{}\n", END_META_4_GROUPS);
    let mut iter = StuckThreadStream(input.as_bytes()).into_iter();
    let result = iter.next();
    assert!(result.is_some(), "failed: {result:?}");
    let chunk = result.unwrap();
    let event = StuckThread::try_from(chunk);
    assert!(event.is_ok(), "Error parsing chunk {chunk:?} : {event:?}");
    let event = event.unwrap();
    println!("{event:?}");
    match event.meta {
        StuckThreadMeta::End(e) => {
            assert_eq!(e.thread_id, 226293);
            assert_eq!(e.active_monitor_count, 78);
            assert_eq!(e.active_duration_ms, 27650);
            assert_eq!(e.thread_name, "");
        }
        _ => {
            panic!("Expected StuckThreadMetaEnd")
        }
    };
}

#[test]
fn stream_parse_multiple_end_records() {
    let input = format!(
        "{}\n{}\n{}\n",
        END_META_4_GROUPS,
        END_META_4_GROUPS_B,
        "[15:52:17.907]|[16-02-2026]|[org.apache.catalina.valves.StuckThreadDetectionValve]|[WARN]|[90]| :: Thread [] (id=[4099]) was previously reported to be stuck but has completed. It was active for approximately [26,330] milliseconds. There is/are still [76] thread(s) that are monitored by this Valve and may be stuck."
    );
    let mut count = 0;
    for chunk in StuckThreadStream(input.as_bytes()) {
        let event = StuckThread::try_from(chunk);
        assert!(event.is_ok());
        let event = event.unwrap();
        let StuckThreadMeta::End(_) = event.meta else {
            panic!("Expected StuckThreadMetaEnd")
        };

        count += 1;
    }
    assert_eq!(count, 3);
}

#[test]
fn stream_parse_begin_then_end() {
    let input = format!("{}{}{}\n", BEGIN_META, SMALL_STACKTRACE, END_META_4_GROUPS);
    let mut count = 0;
    for chunk in StuckThreadStream(input.as_bytes()) {
        let event = StuckThread::try_from(chunk);
        assert!(event.is_ok(), "Parsing {chunk} failed due to : {event:?}");
        let _ = event.unwrap();
        count += 1;
    }
    assert_eq!(count, 2);
}

#[test]
fn stream_parse_empty_input() {
    let result = StuckThreadStream(b"").into_iter().collect::<Vec<_>>().len();
    assert!(result == 0);
}

// ============================================================
// Edge cases — comma-separated numbers, large IDs
// ============================================================

#[test]
fn parse_comma_separated_i64() {
    // Test the helper via StuckThreadMetaBegin
    let groups = vec![
        "thread",
        "999999",
        "1,234,567",
        "since",
        "http://example.com",
        "10",
        "100",
    ];
    let result = StuckThreadMetaBegin::try_from(groups).unwrap();
    assert_eq!(result.active_duration_ms, 1234567);
    assert_eq!(result.thread_id, 999999);
}

#[test]
fn parse_duration_no_comma() {
    let groups = vec![
        "thread",
        "1",
        "500",
        "since",
        "http://example.com",
        "10",
        "1",
    ];
    let result = StuckThreadMetaBegin::try_from(groups).unwrap();
    assert_eq!(result.active_duration_ms, 500);
}

#[test]
fn end_with_empty_thread_name() {
    // Real end records have empty thread names []
    let groups = vec!["", "42", "1,000", "5"];
    let result = StuckThreadMetaEnd::try_from(groups).unwrap();
    assert_eq!(result.thread_name, "");
    assert_eq!(result.thread_id, 42);
}
