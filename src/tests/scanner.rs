use crate::{error::scanner, scanner::Scanner};

// ============================================================
// skip_whitespace
// ============================================================

#[test]
fn scanner_skip_whitespace() -> Result<(), scanner::Error> {
    let mut s = Scanner::new("      Hello");
    s.skip_whitespace();
    assert_eq!(s.data, "Hello");
    Ok(())
}

#[test]
fn scanner_skip_whitespace_ascii() -> Result<(), scanner::Error> {
    let mut s = Scanner::new("\t\t\n\n\r\r      Hello");
    s.skip_whitespace();
    assert_eq!(s.data, "Hello");
    Ok(())
}

#[test]
fn scanner_skip_whitespace_no_whitespace() -> Result<(), scanner::Error> {
    let mut s = Scanner::new("Hello");
    s.skip_whitespace();
    assert_eq!(s.data, "Hello");
    Ok(())
}

#[test]
fn scanner_skip_whitespace_empty_string() -> Result<(), scanner::Error> {
    let mut s = Scanner::new("");
    s.skip_whitespace();
    assert_eq!(s.data, "");
    Ok(())
}

#[test]
fn scanner_skip_whitespace_all_whitespace() -> Result<(), scanner::Error> {
    let mut s = Scanner::new("   \t\n  ");
    s.skip_whitespace();
    assert_eq!(s.data, "");
    Ok(())
}

#[test]
fn scanner_skip_whitespace_unicode() -> Result<(), scanner::Error> {
    // Japanese text after whitespace — trim_start handles it fine
    let mut s = Scanner::new("   処理スレッド");
    s.skip_whitespace();
    assert_eq!(s.data, "処理スレッド");
    Ok(())
}

// ============================================================
// expect
// ============================================================

#[test]
fn scanner_expect_success() -> Result<(), scanner::Error> {
    let mut s = Scanner::new("Id=25 rest");
    s.expect("Id=")?;
    assert_eq!(s.data, "25 rest");
    Ok(())
}

#[test]
fn scanner_expect_entire_input() -> Result<(), scanner::Error> {
    let mut s = Scanner::new("EOF");
    s.expect("EOF")?;
    assert_eq!(s.data, "");
    Ok(())
}

#[test]
fn scanner_expect_empty_prefix() -> Result<(), scanner::Error> {
    let mut s = Scanner::new("Hello");
    s.expect("")?;
    assert_eq!(s.data, "Hello");
    Ok(())
}

#[test]
fn scanner_expect_wrong_prefix() {
    let mut s = Scanner::new("Id=25");
    let result = s.expect("Xd=");
    assert!(result.is_err());
    // data should not have advanced
    assert_eq!(s.data, "Id=25");
}

#[test]
fn scanner_expect_input_too_short() {
    let mut s = Scanner::new("Id");
    let result = s.expect("Id=25");
    assert!(result.is_err());
    assert_eq!(
        result,
        Err(scanner::Error::EndOfData)
    );
}

#[test]
fn scanner_expect_exact_length_mismatch() {
    // Same length as expected, but different content
    let mut s = Scanner::new("abc");
    let result = s.expect("xyz");
    assert!(result.is_err());
    // This exercises the path where len >= expected.len() but prefix doesn't match.
    // NOTE: current impl does [..expected.len() + 1] which would be index 4 on a 3-byte
    // string — this will panic. This test documents that bug.
}

#[test]
fn scanner_expect_chained() -> Result<(), scanner::Error> {
    let mut s = Scanner::new("Java.lang.Thread.State: WAITING");
    s.expect("Java.lang.Thread.State:")?;
    s.skip_whitespace();
    assert_eq!(s.data, "WAITING");
    Ok(())
}

// ============================================================
// take_until
// ============================================================

#[test]
fn scanner_take_until() -> Result<(), scanner::Error> {
    let mut s = Scanner::new("Hello, world");
    let result = s.take_until(",")?;
    assert_eq!("Hello", result);
    assert_eq!(" world", s.data);
    Ok(())
}

#[test]
fn scanner_take_until_pattern_not_found() -> Result<(), scanner::Error> {
    let mut s = Scanner::new("Hello::World");
    let result = s.take_until(",");
    assert!(result.is_err());
    assert_eq!(
        result,
        Err(scanner::Error::DelimiterNotFound {
            delimiter: String::from(","),
            data: String::from("Hello::World")
        })
    );
    Ok(())
}

#[test]
fn scanner_take_until_pattern_entire_string() -> Result<(), scanner::Error> {
    let mut s = Scanner::new("Hello::World");
    let result = s.take_until("Hello::World");
    assert!(result.is_ok(), "GOT: {result:?}");
    assert_eq!(result, Ok(""));
    assert_eq!(s.data, "");
    Ok(())
}

#[test]
fn scanner_take_until_pattern_empty_string() -> Result<(), scanner::Error> {
    let mut s = Scanner::new("Hello::World");
    let result = s.take_until("");
    assert!(result.is_ok(), "GOT: {result:?}");
    assert_eq!(result, Ok(""));
    assert_eq!(s.data, "Hello::World");
    Ok(())
}

#[test]
fn scanner_take_until_multi_char_delimiter() -> Result<(), scanner::Error> {
    let mut s = Scanner::new("key::value::rest");
    let result = s.take_until("::")?;
    assert_eq!(result, "key");
    assert_eq!(s.data, "value::rest");
    Ok(())
}

#[test]
fn scanner_take_until_delimiter_at_start() -> Result<(), scanner::Error> {
    let mut s = Scanner::new(",hello");
    let result = s.take_until(",")?;
    assert_eq!(result, "");
    assert_eq!(s.data, "hello");
    Ok(())
}

#[test]
fn scanner_take_until_delimiter_at_end() -> Result<(), scanner::Error> {
    let mut s = Scanner::new("hello,");
    let result = s.take_until(",")?;
    assert_eq!(result, "hello");
    assert_eq!(s.data, "");
    Ok(())
}

#[test]
fn scanner_take_until_first_occurrence() -> Result<(), scanner::Error> {
    // Should stop at the FIRST delimiter, not the last
    let mut s = Scanner::new("a@b@c");
    let result = s.take_until("@")?;
    assert_eq!(result, "a");
    assert_eq!(s.data, "b@c");
    Ok(())
}

#[test]
fn scanner_take_until_on_empty_input() {
    let mut s = Scanner::new("");
    let result = s.take_until(",");
    assert!(result.is_err());
}

#[test]
fn scanner_take_until_unicode_content() -> Result<(), scanner::Error> {
    // Unicode content between ASCII delimiters
    let mut s = Scanner::new("\"処理スレッド\" rest");
    s.expect("\"")?;
    let name = s.take_until("\"")?;
    assert_eq!(name, "処理スレッド");
    assert_eq!(s.data, " rest");
    Ok(())
}

// ============================================================
// take
// ============================================================

#[test]
fn scanner_take_basic() -> Result<(), scanner::Error> {
    let mut s = Scanner::new("abcdef");
    let result = s.take(3)?;
    assert_eq!(result, "abc");
    assert_eq!(s.data, "def");
    Ok(())
}

#[test]
fn scanner_take_zero() -> Result<(), scanner::Error> {
    let mut s = Scanner::new("hello");
    let result = s.take(0)?;
    assert_eq!(result, "");
    assert_eq!(s.data, "hello");
    Ok(())
}

#[test]
fn scanner_take_exceeds_length() {
    let mut s = Scanner::new("hi");
    let result = s.take(10);
    assert!(result.is_err());
    assert_eq!(
        result,
        Err(scanner::Error::NotEnoughData {
            have: 2,
            expect: 10,
        })
    );
}

#[test]
fn scanner_take_exact_length() {
    // NOTE: current impl uses `>=` so taking exactly all data fails.
    // This test documents that behavior. If you change to `>`, flip the assertion.
    let mut s = Scanner::new("hello");
    let result = s.take(5);
    assert!(result.is_err(), "take(n) where n == len currently fails due to >= check");
}

#[test]
fn scanner_take_hex_id() -> Result<(), scanner::Error> {
    // Realistic: taking an 8-char hex object ID
    let mut s = Scanner::new("73853f10 rest");
    let hex = s.take(8)?;
    assert_eq!(hex, "73853f10");
    assert_eq!(s.data, " rest");
    Ok(())
}

// ============================================================
// peek_until
// ============================================================

#[test]
fn scanner_peek_until_does_not_advance() {
    let mut s = Scanner::new("Hello, World");
    let peeked = s.peek_until(",");
    assert_eq!(peeked, Some("Hello"));
    // data should NOT have changed
    assert_eq!(s.data, "Hello, World");
}

#[test]
fn scanner_peek_until_not_found() {
    let mut s = Scanner::new("Hello World");
    let peeked = s.peek_until(",");
    assert_eq!(peeked, None);
    assert_eq!(s.data, "Hello World");
}

#[test]
fn scanner_peek_until_then_take_until() -> Result<(), scanner::Error> {
    // Peek first to decide, then consume
    let mut s = Scanner::new("WAITING on java.lang.Object@12345678");
    let peeked = s.peek_until(" on ");
    assert_eq!(peeked, Some("WAITING"));

    let state = s.take_until(" on ")?;
    assert_eq!(state, "WAITING");
    assert_eq!(s.data, "java.lang.Object@12345678");
    Ok(())
}

// ============================================================
// take_within
// ============================================================

#[test]
fn scanner_take_within_basic() -> Result<(), scanner::Error> {
    let mut s = Scanner::new("(DataSource.java:359)");
    let result = s.take_within("(", ")")?;
    assert_eq!(result, "DataSource.java:359");
    assert_eq!(s.data, "");
    Ok(())
}

#[test]
fn scanner_take_within_with_surrounding() -> Result<(), scanner::Error> {
    let mut s = Scanner::new("method(Native Method) rest");
    let result = s.take_within("(", ")")?;
    assert_eq!(result, "Native Method");
    assert_eq!(s.data, " rest");
    Ok(())
}

#[test]
fn scanner_take_within_missing_open() {
    let mut s = Scanner::new("no parens here");
    let result = s.take_within("(", ")");
    assert!(result.is_err());
    // data should not advance on error
    assert_eq!(s.data, "no parens here");
}

#[test]
fn scanner_take_within_missing_close() {
    let mut s = Scanner::new("(unclosed");
    let result = s.take_within("(", ")");
    assert!(result.is_err());
}

#[test]
fn scanner_take_within_empty_content() -> Result<(), scanner::Error> {
    let mut s = Scanner::new("()rest");
    let result = s.take_within("(", ")")?;
    assert_eq!(result, "");
    assert_eq!(s.data, "rest");
    Ok(())
}

#[test]
fn scanner_take_within_multi_char_delimiters() -> Result<(), scanner::Error> {
    let mut s = Scanner::new("<<value>>rest");
    let result = s.take_within("<<", ">>")?;
    assert_eq!(result, "value");
    assert_eq!(s.data, "rest");
    Ok(())
}

#[test]
fn scanner_take_within_quotes() -> Result<(), scanner::Error> {
    let mut s = Scanner::new("\"Glowroot-Background-0\"  Id=25");
    let name = s.take_within("\"", "\"")?;
    assert_eq!(name, "Glowroot-Background-0");
    assert_eq!(s.data, "  Id=25");
    Ok(())
}

#[test]
fn scanner_take_within_skips_prefix() {
    // NOTE: take_within silently discards content before `open`.
    // "prefix" is lost. This test documents that behavior.
    let mut s = Scanner::new("prefix(value)rest");
    let result = s.take_within("(", ")").unwrap();
    assert_eq!(result, "value");
    assert_eq!(s.data, "rest");
    // "prefix" was silently consumed — no way to recover it
}

// ============================================================
// Chained operations — realistic thread dump parsing
// ============================================================

#[test]
fn scanner_parse_thread_header_waiting() -> Result<(), scanner::Error> {
    let input = "\"Glowroot-Background-0\"  Id=25  Java.lang.Thread.State: WAITING on java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject@40618d59";
    let mut s = Scanner::new(input);

    let name = s.take_within("\"", "\"")?;
    assert_eq!(name, "Glowroot-Background-0");

    s.skip_whitespace();
    s.expect("Id=")?;
    let id_str = s.take_until(" ")?;
    assert_eq!(id_str, "25");

    s.skip_whitespace();
    s.expect("Java.lang.Thread.State:")?;
    s.skip_whitespace();

    let state = s.take_until(" on ")?;
    assert_eq!(state, "WAITING");

    let object = s.data;
    assert_eq!(
        object,
        "java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject@40618d59"
    );

    Ok(())
}

#[test]
fn scanner_parse_thread_header_blocked() -> Result<(), scanner::Error> {
    let input = "\"Glowroot-Trace-Collector\"  Id=26  Java.lang.Thread.State: BLOCKED waiting to lock java.lang.Object@73853f10";
    let mut s = Scanner::new(input);

    let name = s.take_within("\"", "\"")?;
    assert_eq!(name, "Glowroot-Trace-Collector");

    s.skip_whitespace();
    s.expect("Id=")?;
    let id_str = s.take_until(" ")?;
    assert_eq!(id_str, "26");

    s.skip_whitespace();
    s.expect("Java.lang.Thread.State:")?;
    s.skip_whitespace();

    let state = s.take_until(" waiting to lock ")?;
    assert_eq!(state, "BLOCKED");

    let object = s.data;
    assert_eq!(object, "java.lang.Object@73853f10");

    Ok(())
}

#[test]
fn scanner_parse_lock_metadata() -> Result<(), scanner::Error> {
    let input = "LockName: java.lang.Object@73853f10 Owner Id: 28 Owner Name: Glowroot-Aggregate-Flushing";
    let mut s = Scanner::new(input);

    s.expect("LockName: ")?;
    let lock_obj = s.take_until(" Owner Id: ")?;
    assert_eq!(lock_obj, "java.lang.Object@73853f10");

    let owner_id = s.take_until(" Owner Name: ")?;
    assert_eq!(owner_id, "28");

    let owner_name = s.data;
    assert_eq!(owner_name, "Glowroot-Aggregate-Flushing");

    Ok(())
}

#[test]
fn scanner_parse_stack_frame_with_module() -> Result<(), scanner::Error> {
    let input = "java.base@17.0.17/jdk.internal.misc.Unsafe.park(Native Method)";
    let mut s = Scanner::new(input);

    let frame = s.take_until_inclusive("(")?;
    assert_eq!(frame, "java.base@17.0.17/jdk.internal.misc.Unsafe.park");

    let source = s.take_within("(", ")")?;
    assert_eq!(source, "Native Method");

    Ok(())
}

#[test]
fn scanner_parse_stack_frame_with_source() -> Result<(), scanner::Error> {
    let input = "org.glowroot.agent.embedded.util.DataSource.update(DataSource.java:359)";
    let mut s = Scanner::new(input);

    let frame = s.take_until_inclusive("(")?;
    assert_eq!(frame, "org.glowroot.agent.embedded.util.DataSource.update");

    let source = s.take_within("(", ")")?;
    assert_eq!(source, "DataSource.java:359");

    Ok(())
}
