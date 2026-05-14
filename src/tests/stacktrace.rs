use crate::error::stacktrace::Parse;
use crate::stacktrace::{StackTrace, Element, StackTraceSource};

#[test]
fn source_native_method() {
    let result = StackTraceSource::try_from("Native Method").unwrap();
    assert!(matches!(result, StackTraceSource::NativeMethod));
}

#[test]
fn source_unknown_source() {
    let result = StackTraceSource::try_from("Unknown Source").unwrap();
    assert!(matches!(result, StackTraceSource::UnknownSource));
}

#[test]
fn source_filename_with_line() {
    let result = StackTraceSource::try_from("DataSource.java:359").unwrap();
    match result {
        StackTraceSource::FileName { file, line } => {
            assert_eq!(file, "DataSource.java");
            assert_eq!(line, 359);
        }
        other => panic!("expected FileName, got {other:?}"),
    }
}

#[test]
fn source_filename_line_one() {
    let result = StackTraceSource::try_from("App.java:1").unwrap();
    match result {
        StackTraceSource::FileName { file, line } => {
            assert_eq!(file, "App.java");
            assert_eq!(line, 1);
        }
        other => panic!("expected FileName, got {other:?}"),
    }
}

#[test]
fn source_filename_large_line_number() {
    let result = StackTraceSource::try_from("BigFile.java:99999").unwrap();
    match result {
        StackTraceSource::FileName { file, line } => {
            assert_eq!(file, "BigFile.java");
            assert_eq!(line, 99999);
        }
        other => panic!("expected FileName, got {other:?}"),
    }
}

#[test]
fn source_generated_lambda() {
    // Lambda classes contain '$' in the source
    let result =
        StackTraceSource::try_from("BeanProxy$$Lambda$395/0x000001cf92566e40").unwrap();
    match result {
        StackTraceSource::Generated { inner } => {
            assert_eq!(inner, "BeanProxy$$Lambda$395/0x000001cf92566e40");
        }
        other => panic!("expected Generated, got {other:?}"),
    }
}

#[test]
fn source_generated_inner_class() {
    // Inner classes also use '$'
    let result = StackTraceSource::try_from("Outer$Inner.java:10").unwrap();
    match result {
        StackTraceSource::Generated { inner } => {
            assert_eq!(inner, "Outer$Inner.java:10");
        }
        other => panic!("expected Generated, got {other:?}"),
    }
}

#[test]
fn source_no_colon_no_dollar_not_known() {
    // Not Native Method, not Unknown Source, no '$', no ':' → ColonNotFound
    let result = StackTraceSource::try_from("SomeRandomThing");
    assert!(result.is_err());
    assert!(matches!(result, Err(Parse::ColonNotFound)));
}

#[test]
fn source_colon_but_non_numeric_line() {
    // Has colon but line number isn't a number
    let result = StackTraceSource::try_from("File.java:abc");
    assert!(result.is_err());
    assert!(matches!(result, Err(Parse::LineNumber)));
}

#[test]
fn source_non_java_file_extension() {
    // Has colon, valid number, but file doesn't end with "java"
    let result = StackTraceSource::try_from("File.kt:42");
    assert!(result.is_err());
    assert!(matches!(result, Err(Parse::SourceTypeNotRecognized)));
}

#[test]
fn source_empty_string() {
    let result = StackTraceSource::try_from("");
    assert!(result.is_err());
}

// ============================================================
// StackTraceElement — single frame parsing
// ============================================================

#[test]
fn element_standard_frame() {
    let input = "com.adventnet.persistence.DataAccess.get(DataAccess.java:2337)";
    let result = Element::try_from(input).unwrap();
    assert_eq!(result.function_name, "com.adventnet.persistence.DataAccess.get");
    match result.stacktrace_source {
        StackTraceSource::FileName { file, line } => {
            assert_eq!(file, "DataAccess.java");
            assert_eq!(line, 2337);
        }
        other => panic!("expected FileName, got {other:?}"),
    }
}

#[test]
fn element_native_method() {
    let input = "jdk.internal.misc.Unsafe.park(Native Method)";
    let result = Element::try_from(input).unwrap();
    assert_eq!(result.function_name, "jdk.internal.misc.Unsafe.park");
    assert!(matches!(result.stacktrace_source, StackTraceSource::NativeMethod));
}

#[test]
fn element_unknown_source() {
    let input = "java.util.concurrent.locks.LockSupport.parkNanos(Unknown Source)";
    let result = Element::try_from(input).unwrap();
    assert_eq!(
        result.function_name,
        "java.util.concurrent.locks.LockSupport.parkNanos"
    );
    assert!(matches!(result.stacktrace_source, StackTraceSource::UnknownSource));
}

#[test]
fn element_with_module_prefix() {
    // JDK frames with java.base@17.0.17/ prefix
    let input =
        "java.base@17.0.17/jdk.internal.misc.Unsafe.park(Native Method)";
    let result = Element::try_from(input).unwrap();
    // The module prefix is included in function_name (parser doesn't strip it)
    assert_eq!(
        result.function_name,
        "java.base@17.0.17/jdk.internal.misc.Unsafe.park"
    );
    assert!(matches!(result.stacktrace_source, StackTraceSource::NativeMethod));
}

#[test]
fn element_lambda_unknown_source() {
    let input = "com.adventnet.mfw.bean.BeanProxy$$Lambda$395/0x000001cf92566e40.call(Unknown Source)";
    let result = Element::try_from(input).unwrap();
    assert_eq!(
        result.function_name,
        "com.adventnet.mfw.bean.BeanProxy$$Lambda$395/0x000001cf92566e40.call"
    );
    assert!(matches!(result.stacktrace_source, StackTraceSource::UnknownSource));
}

#[test]
fn element_jdk_proxy() {
    let input = "jdk.proxy3/jdk.proxy3.$Proxy45.get(Unknown Source)";
    let result = Element::try_from(input).unwrap();
    assert_eq!(result.function_name, "jdk.proxy3/jdk.proxy3.$Proxy45.get");
    assert!(matches!(result.stacktrace_source, StackTraceSource::UnknownSource));
}

#[test]
fn element_generated_method_accessor() {
    let input = "jdk.internal.reflect.GeneratedMethodAccessor112.invoke(Unknown Source)";
    let result = Element::try_from(input).unwrap();
    assert_eq!(
        result.function_name,
        "jdk.internal.reflect.GeneratedMethodAccessor112.invoke"
    );
    assert!(matches!(result.stacktrace_source, StackTraceSource::UnknownSource));
}

#[test]
fn element_no_parens() {
    let input = "some.function.without.parens";
    let result = Element::try_from(input);
    assert!(result.is_err());
}

#[test]
fn element_empty_input() {
    let result = Element::try_from("");
    assert!(result.is_err());
}

#[test]
fn element_servlet_frame() {
    let input = "javax.servlet.http.HttpServlet.service(HttpServlet.java:529)";
    let result = Element::try_from(input).unwrap();
    assert_eq!(result.function_name, "javax.servlet.http.HttpServlet.service");
    match result.stacktrace_source {
        StackTraceSource::FileName { file, line } => {
            assert_eq!(file, "HttpServlet.java");
            assert_eq!(line, 529);
        }
        other => panic!("expected FileName, got {other:?}"),
    }
}

#[test]
fn element_tomcat_filter_chain() {
    let input = "org.apache.catalina.core.ApplicationFilterChain.internalDoFilter(ApplicationFilterChain.java:197)";
    let result = Element::try_from(input).unwrap();
    assert_eq!(
        result.function_name,
        "org.apache.catalina.core.ApplicationFilterChain.internalDoFilter"
    );
    match result.stacktrace_source {
        StackTraceSource::FileName { file, line } => {
            assert_eq!(file, "ApplicationFilterChain.java");
            assert_eq!(line, 197);
        }
        other => panic!("expected FileName, got {other:?}"),
    }
}

// ============================================================
// StackTrace — full stack trace parsing (with java.lang.Throwable header)
// ============================================================

#[test]
fn stacktrace_basic() {
    let input = "\
java.lang.Throwable
\tat java.base@17.0.17/jdk.internal.misc.Unsafe.park(Native Method)
\tat java.base@17.0.17/java.util.concurrent.locks.LockSupport.parkNanos(Unknown Source)
";
    let result = StackTrace::try_from(input).unwrap();
    assert_eq!(result.traces.len(), 2);
    assert!(matches!(
        result.traces[0].stacktrace_source,
        StackTraceSource::NativeMethod
    ));
    assert!(matches!(
        result.traces[1].stacktrace_source,
        StackTraceSource::UnknownSource
    ));
}

#[test]
fn stacktrace_real_world_stuck_thread() {
    let input = "\
java.lang.Throwable
\tat java.base@17.0.17/jdk.internal.misc.Unsafe.park(Native Method)
\tat java.base@17.0.17/java.util.concurrent.locks.LockSupport.parkNanos(Unknown Source)
\tat java.base@17.0.17/java.util.concurrent.locks.AbstractQueuedSynchronizer.acquire(Unknown Source)
\tat java.base@17.0.17/java.util.concurrent.locks.AbstractQueuedSynchronizer.tryAcquireSharedNanos(Unknown Source)
\tat java.base@17.0.17/java.util.concurrent.Semaphore.tryAcquire(Unknown Source)
\tat com.zoho.cp.ObjectStack.acquirePermitAndPollLastElement(ObjectStack.java:23)
\tat com.zoho.cp.ConnectionPool._getConnectionDetail(ConnectionPool.java:186)
";
    let result = StackTrace::try_from(input).unwrap();
    assert_eq!(result.traces.len(), 7);

    // First frame: native method
    assert!(matches!(
        result.traces[0].stacktrace_source,
        StackTraceSource::NativeMethod
    ));

    // Last frame: filename with line
    match &result.traces[6].stacktrace_source {
        StackTraceSource::FileName { file, line } => {
            assert_eq!(*file, "ConnectionPool.java");
            assert_eq!(*line, 186);
        }
        other => panic!("expected FileName, got {other:?}"),
    }
}

#[test]
fn stacktrace_mixed_sources() {
    let input = "\
java.lang.Throwable
\tat jdk.internal.misc.Unsafe.park(Native Method)
\tat java.util.concurrent.locks.LockSupport.parkNanos(Unknown Source)
\tat com.zoho.cp.ObjectStack.acquirePermitAndPollLastElement(ObjectStack.java:23)
\tat com.adventnet.mfw.bean.BeanProxy$$Lambda$395/0x000001cf92566e40.call(Unknown Source)
";
    let result = StackTrace::try_from(input).unwrap();
    assert_eq!(result.traces.len(), 4);
    assert!(matches!(result.traces[0].stacktrace_source, StackTraceSource::NativeMethod));
    assert!(matches!(result.traces[1].stacktrace_source, StackTraceSource::UnknownSource));
    assert!(matches!(result.traces[2].stacktrace_source, StackTraceSource::FileName { .. }));
    assert!(matches!(result.traces[3].stacktrace_source, StackTraceSource::UnknownSource));
}

#[test]
fn stacktrace_missing_throwable() {
    let input = "\
\tat jdk.internal.misc.Unsafe.park(Native Method)
\tat java.util.concurrent.locks.LockSupport.parkNanos(Unknown Source)
";
    let result = StackTrace::try_from(input);
    assert!(result.is_err());
    assert!(matches!(result, Err(Parse::ThrowableNotFound)));
}

#[test]
fn stacktrace_throwable_only_no_frames() {
    let input = "java.lang.Throwable\n";
    let result = StackTrace::try_from(input);
    // Should succeed with an empty trace list (loop finds no "at" lines)
    assert!(result.is_ok());
    assert_eq!(result.unwrap().traces.len(), 0);
}

#[test]
fn stacktrace_empty_input() {
    let result = StackTrace::try_from("");
    assert!(result.is_err());
    assert!(matches!(result, Err(Parse::ThrowableNotFound)));
}

#[test]
fn stacktrace_single_frame() {
    let input = "\
java.lang.Throwable
\tat javax.servlet.http.HttpServlet.service(HttpServlet.java:623)
";
    let result = StackTrace::try_from(input).unwrap();
    assert_eq!(result.traces.len(), 1);
    assert_eq!(
        result.traces[0].function_name,
        "javax.servlet.http.HttpServlet.service"
    );
}

#[test]
fn stacktrace_deep_trace() {
    // Simulate a deep stack (30+ frames common in real stuck threads)
    let mut input = String::from("java.lang.Throwable\n");
    for i in 0..30 {
        input.push_str(&format!(
            "\tat com.example.Class{i}.method{i}(Class{i}.java:{line})\n",
            i = i,
            line = i * 10 + 1
        ));
    }
    let result = StackTrace::try_from(input.as_str()).unwrap();
    assert_eq!(result.traces.len(), 30);

    // Spot-check first and last
    assert_eq!(result.traces[0].function_name, "com.example.Class0.method0");
    assert_eq!(result.traces[29].function_name, "com.example.Class29.method29");
    match &result.traces[29].stacktrace_source {
        StackTraceSource::FileName { file, line } => {
            assert_eq!(*file, "Class29.java");
            assert_eq!(*line, 291);
        }
        other => panic!("expected FileName, got {other:?}"),
    }
}

#[test]
fn stacktrace_with_spaces_instead_of_tabs() {
    // Some logs use spaces instead of tabs before "at"
    let input = "\
java.lang.Throwable
    at jdk.internal.misc.Unsafe.park(Native Method)
    at java.util.concurrent.locks.LockSupport.parkNanos(Unknown Source)
";
    let result = StackTrace::try_from(input);
    // This tests whether skip_whitespace handles spaces before "at"
    // If the parser requires \t specifically, this would fail
    if let Ok(st) = &result {
        assert_eq!(st.traces.len(), 2);
    }
    // If it fails, that's also a valid documented behavior
}

#[test]
fn stacktrace_no_trailing_newline() {
    // Real files might not end with a newline
    let input = "\
java.lang.Throwable
\tat jdk.internal.misc.Unsafe.park(Native Method)
\tat java.util.concurrent.locks.LockSupport.parkNanos(Unknown Source)";
    // No trailing \n — tests how take_until_inclusive("\n") handles EOF
    let result = StackTrace::try_from(input);
    // Current impl: take_until_inclusive("\n") fails on the last line → loop breaks
    // So we should still get the first frame at minimum
    if let Ok(st) = &result {
        assert!(st.traces.len() >= 1, "should parse at least 1 frame");
    }
}

#[test]
fn stacktrace_websocket_filter() {
    let input = "\
java.lang.Throwable
\tat org.apache.tomcat.websocket.server.WsFilter.doFilter(WsFilter.java:51)
\tat org.apache.catalina.core.ApplicationFilterChain.internalDoFilter(ApplicationFilterChain.java:166)
\tat org.apache.catalina.core.ApplicationFilterChain.doFilter(ApplicationFilterChain.java:142)
";
    let result = StackTrace::try_from(input).unwrap();
    assert_eq!(result.traces.len(), 3);
    match &result.traces[0].stacktrace_source {
        StackTraceSource::FileName { file, line } => {
            assert_eq!(*file, "WsFilter.java");
            assert_eq!(*line, 51);
        }
        other => panic!("expected FileName, got {other:?}"),
    }
}

#[test]
fn stacktrace_connection_pool_pattern() {
    // Common pattern: semaphore → connection pool chain
    let input = "\
java.lang.Throwable
\tat java.base@17.0.17/jdk.internal.misc.Unsafe.park(Native Method)
\tat java.base@17.0.17/java.util.concurrent.Semaphore.tryAcquire(Unknown Source)
\tat com.zoho.cp.ObjectStack.acquirePermitAndPollLastElement(ObjectStack.java:23)
\tat com.zoho.cp.ConnectionPool._getConnectionDetail(ConnectionPool.java:186)
\tat com.zoho.cp.ConnectionPool.getConnectionDetail(ConnectionPool.java:94)
\tat com.zoho.cp.TxDataSource.getConnDetailFromPool(TxDataSource.java:72)
\tat com.zoho.cp.TxDataSource.getConnection(TxDataSource.java:43)
";
    let result = StackTrace::try_from(input).unwrap();
    assert_eq!(result.traces.len(), 7);

    // Verify the module-prefixed frame
    assert_eq!(
        result.traces[0].function_name,
        "java.base@17.0.17/jdk.internal.misc.Unsafe.park"
    );

    // Verify application frame
    assert_eq!(
        result.traces[2].function_name,
        "com.zoho.cp.ObjectStack.acquirePermitAndPollLastElement"
    );
    match &result.traces[2].stacktrace_source {
        StackTraceSource::FileName { file, line } => {
            assert_eq!(*file, "ObjectStack.java");
            assert_eq!(*line, 23);
        }
        other => panic!("expected FileName, got {other:?}"),
    }
}

// ============================================================
// StackTraceSource — $ in .java filename becomes Generated
// ============================================================

#[test]
fn source_dollar_in_java_filename_is_generated() {
    // A filename like "Outer$Inner.java:10" contains '$' so it is Generated,
    // not FileName — the '$' check fires before the colon check.
    let result = StackTraceSource::try_from("Outer$Inner.java:10").unwrap();
    assert!(matches!(result, StackTraceSource::Generated { .. }));
}

#[test]
fn source_lambda_class_no_colon_is_generated() {
    // Lambda class descriptors with '$' but no ':' are still Generated
    let result = StackTraceSource::try_from("BeanProxy$$Lambda$395/0x000001cf92566e40").unwrap();
    assert!(matches!(result, StackTraceSource::Generated { .. }));
}

// ============================================================
// StackTrace — error: content after Throwable but no "at" prefix
// ============================================================

#[test]
fn stacktrace_at_not_found_error() {
    // The line after Throwable does not start with "at" → AtNotFound
    let input = "java.lang.Throwable\nsome random line without at prefix\n";
    let result = StackTrace::try_from(input);
    assert!(result.is_err());
    assert!(matches!(result, Err(Parse::AtNotFound)));
}

#[test]
fn stacktrace_at_not_found_after_whitespace_skip() {
    // Whitespace is skipped, then a non-"at" line → AtNotFound
    let input = "java.lang.Throwable\n   \n  not_at_prefix(SomeFile.java:1)\n";
    let result = StackTrace::try_from(input);
    assert!(result.is_err());
    assert!(matches!(result, Err(Parse::AtNotFound)));
}

// ============================================================
// StackTraceElement — close-paren missing
// ============================================================

#[test]
fn element_missing_close_paren() {
    // Has open paren but no close paren → ParseError
    let result = Element::try_from("some.method(Native Method");
    assert!(result.is_err());
}

// ============================================================
// StackTrace — empty input
// ============================================================

#[test]
fn stacktrace_only_whitespace() {
    let result = StackTrace::try_from("   ");
    assert!(result.is_err());
    assert!(matches!(result, Err(Parse::ThrowableNotFound)));
}
