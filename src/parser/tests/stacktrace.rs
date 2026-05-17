//! Tests for `crate::parser::stacktrace`.
//!
//! Sections:
//!   1.  `Source::try_from`          — leaf source parsing
//!   2.  `Object::hex_to_u64`        — hex byte parsing
//!   3.  `Object::try_from`          — `class@identity` parsing
//!   4.  `Element::try_from`         — single line (frame or lock)
//!   5.  `Trace::try_from`           — full driver loop
//!   6.  Lifetimes / zero-copy       — returned slices borrow from input
//!   7.  Smoke / fuzz-ish            — non-panic guarantees on arbitrary input
//!
//! Bug markers used below:
//!
//!   BUG-1 : `Source::Filename` slices `:` onto the line bytes
//!           (`split_at(pos)` instead of `split_at(pos + 1)`).
//!           Every "happy path" Filename test will fail until fixed.
//!
//!   BUG-2 : `Trace::try_from` cannot parse the canonical Java header
//!           `java.lang.Throwable: <message>` — it requires the next
//!           non-whitespace bytes to be `at`.
//!
//!   BUG-3 : `Trace::try_from` cannot parse `- locked` lines interleaved
//!           with frame lines, because the driver `expect(b"at")` on
//!           every iteration. `Element::try_from` handles locks fine
//!           when called directly.

use crate::error::stacktrace::Parse;
use crate::parser::stacktrace::{Element, Object, Source, Trace};

// ============================================================================
// 1. Source::try_from
// ============================================================================

// --- exact-match branch ------------------------------------------------------

#[test]
fn source_exact_unknown_source() {
    let r = Source::try_from(b"Unknown Source".as_slice()).unwrap();
    assert!(matches!(r, Source::UnknownSource));
}

#[test]
fn source_exact_native_method() {
    let r = Source::try_from(b"Native Method".as_slice()).unwrap();
    assert!(matches!(r, Source::NativeMethod));
}

#[test]
fn source_case_sensitive_native_method_lower_falls_to_generated() {
    // "native method" lowercase: no exact match, no `:` -> Generated
    let r = Source::try_from(b"native method".as_slice()).unwrap();
    assert!(matches!(r, Source::Generated(b"native method")));
}

#[test]
fn source_case_sensitive_unknown_source_lower_falls_to_generated() {
    let r = Source::try_from(b"unknown source".as_slice()).unwrap();
    assert!(matches!(r, Source::Generated(b"unknown source")));
}

#[test]
fn source_native_method_trailing_extra_byte_is_generated() {
    let r = Source::try_from(b"Native Methods".as_slice()).unwrap();
    assert!(matches!(r, Source::Generated(b"Native Methods")));
}

#[test]
fn source_native_method_leading_extra_byte_is_generated() {
    let r = Source::try_from(b" Native Method".as_slice()).unwrap();
    assert!(matches!(r, Source::Generated(b" Native Method")));
}

#[test]
fn source_native_method_with_trailing_whitespace_is_generated() {
    let r = Source::try_from(b"Native Method ".as_slice()).unwrap();
    assert!(matches!(r, Source::Generated(b"Native Method ")));
}

// --- generated branch (no `:`) ----------------------------------------------

#[test]
fn source_generated_lambda_short() {
    let r = Source::try_from(b"Lambda$10".as_slice()).unwrap();
    assert!(matches!(r, Source::Generated(b"Lambda$10")));
}

#[test]
fn source_generated_lambda_descriptor() {
    let input = b"BeanProxy$$Lambda$395/0x000001cf92566e40".as_slice();
    match Source::try_from(input).unwrap() {
        Source::Generated(inner) => assert_eq!(inner, input),
        other => panic!("expected Generated, got {other:?}"),
    }
}

#[test]
fn source_generated_empty_input_is_generated_empty() {
    match Source::try_from(b"".as_slice()).unwrap() {
        Source::Generated(b"") => {}
        other => panic!("expected Generated(b\"\"), got {other:?}"),
    }
}

#[test]
fn source_generated_lots_of_dollars() {
    let input = b"$$$$$$".as_slice();
    assert!(matches!(
        Source::try_from(input).unwrap(),
        Source::Generated(b"$$$$$$")
    ));
}

#[test]
fn source_generated_non_ascii_bytes_preserved() {
    let input: &[u8] = &[0xE6, 0x97, 0xA5, 0xE6, 0x9C, 0xAC, 0xE8, 0xAA, 0x9E]; // 日本語
    match Source::try_from(input).unwrap() {
        Source::Generated(inner) => assert_eq!(inner, input),
        other => panic!("expected Generated, got {other:?}"),
    }
}

#[test]
fn source_generated_large_input_no_panic() {
    let big = vec![b'a'; 8192];
    match Source::try_from(big.as_slice()).unwrap() {
        Source::Generated(inner) => assert_eq!(inner.len(), 8192),
        other => panic!("expected Generated, got {other:?}"),
    }
}

// --- filename branch (`:` present) -------------------------------------------
// All marked BUG-1 — will pass once `split_at(pos + 1)` is applied.

#[test]
fn source_filename_basic() {
    // BUG-1
    let r = Source::try_from(b"NioEndpoint.java:10".as_slice()).unwrap();
    match r {
        Source::Filename { file, line } => {
            assert_eq!(file, b"NioEndpoint.java");
            assert_eq!(line, 10);
        }
        other => panic!("expected Filename, got {other:?}"),
    }
}

#[test]
fn source_filename_line_one() {
    // BUG-1
    match Source::try_from(b"App.java:1".as_slice()).unwrap() {
        Source::Filename { file, line } => {
            assert_eq!(file, b"App.java");
            assert_eq!(line, 1);
        }
        other => panic!("expected Filename, got {other:?}"),
    }
}

#[test]
fn source_filename_line_zero() {
    // BUG-1
    match Source::try_from(b"App.java:0".as_slice()).unwrap() {
        Source::Filename { file, line } => {
            assert_eq!(file, b"App.java");
            assert_eq!(line, 0);
        }
        other => panic!("expected Filename, got {other:?}"),
    }
}

#[test]
fn source_filename_large_line() {
    // BUG-1
    match Source::try_from(b"BigFile.java:99999".as_slice()).unwrap() {
        Source::Filename { file, line } => {
            assert_eq!(file, b"BigFile.java");
            assert_eq!(line, 99999);
        }
        other => panic!("expected Filename, got {other:?}"),
    }
}

#[test]
fn source_filename_u64_max() {
    // BUG-1
    let s = format!("Big.java:{}", u64::MAX);
    match Source::try_from(s.as_bytes()).unwrap() {
        Source::Filename { line, .. } => assert_eq!(line, u64::MAX),
        other => panic!("expected Filename, got {other:?}"),
    }
}

#[test]
fn source_filename_leading_zeros_line() {
    // BUG-1
    match Source::try_from(b"App.java:0007".as_slice()).unwrap() {
        Source::Filename { line, .. } => assert_eq!(line, 7),
        other => panic!("expected Filename, got {other:?}"),
    }
}

#[test]
fn source_filename_non_java_extension_accepted() {
    // This parser does NOT enforce `.java`. BUG-1.
    match Source::try_from(b"File.kt:42".as_slice()).unwrap() {
        Source::Filename { file, line } => {
            assert_eq!(file, b"File.kt");
            assert_eq!(line, 42);
        }
        other => panic!("expected Filename, got {other:?}"),
    }
}

#[test]
fn source_filename_no_extension_accepted() {
    // BUG-1
    match Source::try_from(b"App:42".as_slice()).unwrap() {
        Source::Filename { file, line } => {
            assert_eq!(file, b"App");
            assert_eq!(line, 42);
        }
        other => panic!("expected Filename, got {other:?}"),
    }
}

#[test]
fn source_filename_empty_file_part() {
    // BUG-1 — file = b"", line = 42
    match Source::try_from(b":42".as_slice()).unwrap() {
        Source::Filename { file, line } => {
            assert_eq!(file, b"");
            assert_eq!(line, 42);
        }
        other => panic!("expected Filename, got {other:?}"),
    }
}

// --- filename branch errors --------------------------------------------------

#[test]
fn source_filename_empty_line_part_errors() {
    assert!(Source::try_from(b"File.java:".as_slice()).is_err());
}

#[test]
fn source_filename_non_numeric_line_errors() {
    assert!(Source::try_from(b"File.java:abc".as_slice()).is_err());
}

#[test]
fn source_filename_trailing_garbage_errors() {
    assert!(Source::try_from(b"File.java:42x".as_slice()).is_err());
}

#[test]
fn source_filename_negative_line_errors() {
    // Parsed as u64; '-' is not a valid digit.
    assert!(Source::try_from(b"File.java:-1".as_slice()).is_err());
}

#[test]
fn source_filename_floating_point_line_errors() {
    assert!(Source::try_from(b"File.java:1.5".as_slice()).is_err());
}

#[test]
fn source_filename_overflow_u64_errors() {
    // 30 digits overflows u64 ( max ~1.8e19, 20 digits ).
    assert!(Source::try_from(b"File.java:999999999999999999999999999999".as_slice()).is_err());
}

#[test]
fn source_filename_multiple_colons_keeps_first() {
    // memchr finds the FIRST `:` -> line bytes = "10:20" -> error
    assert!(Source::try_from(b"App.java:10:20".as_slice()).is_err());
}

#[test]
fn source_filename_only_colon_errors() {
    // file = b"", line = b"" (after the fix) -> ParseInt error
    assert!(Source::try_from(b":".as_slice()).is_err());
}

#[test]
fn source_filename_whitespace_in_line_errors() {
    // BUG-1 aside, " 42" with leading space isn't a valid u64.
    assert!(Source::try_from(b"App.java: 42".as_slice()).is_err());
}

// ============================================================================
// 2. Object::hex_to_u64
// ============================================================================

#[test]
fn hex_lowercase_basic() {
    assert_eq!(Object::hex_to_u64(b"199d244d").unwrap(), 0x199d_244d);
}

#[test]
fn hex_zero() {
    assert_eq!(Object::hex_to_u64(b"0").unwrap(), 0);
}

#[test]
fn hex_zero_padded() {
    assert_eq!(Object::hex_to_u64(b"00000000").unwrap(), 0);
}

#[test]
fn hex_single_digit_decimal() {
    assert_eq!(Object::hex_to_u64(b"9").unwrap(), 9);
}

#[test]
fn hex_single_digit_alpha_lower() {
    assert_eq!(Object::hex_to_u64(b"a").unwrap(), 10);
    assert_eq!(Object::hex_to_u64(b"f").unwrap(), 15);
}

#[test]
fn hex_single_digit_alpha_upper() {
    assert_eq!(Object::hex_to_u64(b"A").unwrap(), 10);
    assert_eq!(Object::hex_to_u64(b"F").unwrap(), 15);
}

#[test]
fn hex_mixed_case() {
    assert_eq!(Object::hex_to_u64(b"DeAdBeEf").unwrap(), 0xdead_beef);
}

#[test]
fn hex_u32_max() {
    assert_eq!(Object::hex_to_u64(b"ffffffff").unwrap(), 0xffff_ffff);
}

#[test]
fn hex_u64_max_16_digits() {
    assert_eq!(Object::hex_to_u64(b"ffffffffffffffff").unwrap(), u64::MAX);
}

#[test]
fn hex_overflow_17_digits() {
    assert!(Object::hex_to_u64(b"10000000000000000").is_err());
}

#[test]
fn hex_invalid_digit_x() {
    match Object::hex_to_u64(b"199d244x").unwrap_err() {
        Parse::HexUnexpectedInput { got } => assert_eq!(got, "199d244x"),
        other => panic!("expected HexUnexpectedInput, got {other:?}"),
    }
}

#[test]
fn hex_invalid_digit_g() {
    assert!(Object::hex_to_u64(b"deadbegg").is_err());
}

#[test]
fn hex_leading_0x_prefix_invalid() {
    // `u64::from_str_radix` rejects the "0x" prefix.
    assert!(Object::hex_to_u64(b"0xdeadbeef").is_err());
}

#[test]
fn hex_leading_whitespace_invalid() {
    // hex_to_u64 does NOT trim — leading space is an invalid digit.
    assert!(Object::hex_to_u64(b" deadbeef").is_err());
}

#[test]
fn hex_trailing_whitespace_invalid() {
    assert!(Object::hex_to_u64(b"deadbeef ").is_err());
}

#[test]
fn hex_empty_errors() {
    assert!(Object::hex_to_u64(b"").is_err());
}

#[test]
fn hex_non_utf8_errors_with_utf8_conversion() {
    // 0xFF is not valid UTF-8 -> Parse::UTF8Conversion (via `#[from] Utf8Error`)
    let result = Object::hex_to_u64(&[0xFF]);
    assert!(matches!(result, Err(Parse::UTF8Conversion(_))));
}

#[test]
fn hex_non_utf8_continuation_byte_errors() {
    let result = Object::hex_to_u64(&[0x80]); // lone continuation byte
    assert!(matches!(result, Err(Parse::UTF8Conversion(_))));
}

// ============================================================================
// 3. Object::try_from
// ============================================================================

#[test]
fn object_basic() {
    let r = Object::try_from(b"java.lang.Object@73853f10".as_slice()).unwrap();
    assert_eq!(r.class, b"java.lang.Object");
    assert_eq!(r.identity, 0x7385_3f10);
}

#[test]
fn object_inner_class() {
    let input =
        b"java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject@586806fa"
            .as_slice();
    let r = Object::try_from(input).unwrap();
    assert_eq!(
        r.class,
        b"java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject"
    );
    assert_eq!(r.identity, 0x5868_06fa);
}

#[test]
fn object_array_class() {
    // JVM array-type descriptor `[B` for `byte[]`
    let r = Object::try_from(b"[B@cafebabe".as_slice()).unwrap();
    assert_eq!(r.class, b"[B");
    assert_eq!(r.identity, 0xcafe_babe);
}

#[test]
fn object_dotless_class() {
    let r = Object::try_from(b"Foo@1".as_slice()).unwrap();
    assert_eq!(r.class, b"Foo");
    assert_eq!(r.identity, 1);
}

#[test]
fn object_class_with_dollar() {
    let r = Object::try_from(b"Outer$Inner@deadbeef".as_slice()).unwrap();
    assert_eq!(r.class, b"Outer$Inner");
    assert_eq!(r.identity, 0xdead_beef);
}

#[test]
fn object_leading_whitespace_stripped() {
    let r = Object::try_from(b"   \t java.lang.Object@1".as_slice()).unwrap();
    assert_eq!(r.class, b"java.lang.Object");
    assert_eq!(r.identity, 1);
}

#[test]
fn object_only_whitespace_then_class() {
    let r = Object::try_from(b"\n\r\t java.lang.Object@1".as_slice()).unwrap();
    assert_eq!(r.class, b"java.lang.Object");
    assert_eq!(r.identity, 1);
}

#[test]
fn object_empty_class() {
    let r = Object::try_from(b"@73853f10".as_slice()).unwrap();
    assert_eq!(r.class, b"");
    assert_eq!(r.identity, 0x7385_3f10);
}

#[test]
fn object_missing_at() {
    assert!(matches!(
        Object::try_from(b"java.lang.Object-73853f10".as_slice()),
        Err(Parse::MissingCommat)
    ));
}

#[test]
fn object_empty_input() {
    assert!(matches!(
        Object::try_from(b"".as_slice()),
        Err(Parse::MissingCommat)
    ));
}

#[test]
fn object_only_whitespace() {
    assert!(matches!(
        Object::try_from(b"   ".as_slice()),
        Err(Parse::MissingCommat)
    ));
}

#[test]
fn object_empty_identity_errors() {
    assert!(Object::try_from(b"java.lang.Object@".as_slice()).is_err());
}

#[test]
fn object_module_prefixed_takes_first_at() {
    // `take_until(b"@")` finds FIRST `@` — module names with `@` will mis-parse.
    // Documents that this parser expects raw `class@hex`, not `module@ver/class@hex`.
    let input = b"java.base@17.0.17/java.lang.Object@deadbeef".as_slice();
    let r = Object::try_from(input);
    assert!(r.is_err(), "module-prefixed class should fail: {r:?}");
}

#[test]
fn object_double_at_errors() {
    // class = "x", identity = "@deadbeef" — `@` not a valid hex digit
    assert!(Object::try_from(b"x@@deadbeef".as_slice()).is_err());
}

#[test]
fn object_class_with_internal_space_preserved() {
    // Unrealistic but well-defined: anything before the `@` is the class.
    let r = Object::try_from(b"weird class name@1".as_slice()).unwrap();
    assert_eq!(r.class, b"weird class name");
    assert_eq!(r.identity, 1);
}

// ============================================================================
// 4. Element::try_from
// ============================================================================

// --- lock branch -------------------------------------------------------------

#[test]
fn element_lock_basic() {
    let r = Element::try_from(b"- locked java.io.BufferedInputStream@50ef4efc".as_slice())
        .unwrap();
    match r {
        Element::Lock(obj) => {
            assert_eq!(obj.class, b"java.io.BufferedInputStream");
            assert_eq!(obj.identity, 0x50ef_4efc);
        }
        other => panic!("expected Lock, got {other:?}"),
    }
}

#[test]
fn element_lock_with_leading_tab() {
    let r = Element::try_from(b"\t- locked java.lang.Object@1".as_slice()).unwrap();
    assert!(matches!(r, Element::Lock(_)));
}

#[test]
fn element_lock_with_leading_spaces() {
    let r = Element::try_from(b"    - locked java.lang.Object@1".as_slice()).unwrap();
    assert!(matches!(r, Element::Lock(_)));
}

#[test]
fn element_lock_with_mixed_leading_whitespace() {
    let r = Element::try_from(b" \t \t - locked java.lang.Object@1".as_slice()).unwrap();
    assert!(matches!(r, Element::Lock(_)));
}

#[test]
fn element_lock_with_array_type() {
    let r = Element::try_from(b"- locked [I@cafebabe".as_slice()).unwrap();
    match r {
        Element::Lock(obj) => {
            assert_eq!(obj.class, b"[I");
            assert_eq!(obj.identity, 0xcafe_babe);
        }
        other => panic!("expected Lock, got {other:?}"),
    }
}

#[test]
fn element_lock_uppercase_does_not_match() {
    // "- LOCKED" — case-sensitive — falls through to frame parsing,
    // which has no `(` -> ParenNotFound
    let r = Element::try_from(b"- LOCKED Foo@1".as_slice());
    assert!(matches!(r, Err(Parse::ParenNotFound)));
}

#[test]
fn element_lock_capitalized_does_not_match() {
    let r = Element::try_from(b"- Locked Foo@1".as_slice());
    assert!(matches!(r, Err(Parse::ParenNotFound)));
}

#[test]
fn element_lock_no_space_after_dash_does_not_match() {
    let r = Element::try_from(b"-locked Foo@1".as_slice());
    assert!(matches!(r, Err(Parse::ParenNotFound)));
}

#[test]
fn element_lock_keyword_only_then_eof_errors() {
    // peek matches "- locked", then expect consumes it, then skip_ws,
    // then Object::try_from on empty -> MissingCommat
    assert!(matches!(
        Element::try_from(b"- locked".as_slice()),
        Err(Parse::MissingCommat)
    ));
}

#[test]
fn element_lock_keyword_then_whitespace_only_errors() {
    assert!(matches!(
        Element::try_from(b"- locked   \t   ".as_slice()),
        Err(Parse::MissingCommat)
    ));
}

#[test]
fn element_lock_object_without_at_errors() {
    assert!(matches!(
        Element::try_from(b"- locked NoCommatHere".as_slice()),
        Err(Parse::MissingCommat)
    ));
}

#[test]
fn element_lock_object_with_invalid_hex_errors() {
    assert!(Element::try_from(b"- locked Foo@zz".as_slice()).is_err());
}

// --- frame branch ------------------------------------------------------------

#[test]
fn element_frame_native_method() {
    let r = Element::try_from(b"jdk.internal.misc.Unsafe.park(Native Method)".as_slice())
        .unwrap();
    match r {
        Element::Elem { method: frame, source } => {
            assert_eq!(frame, b"jdk.internal.misc.Unsafe.park");
            assert!(matches!(source, Source::NativeMethod));
        }
        other => panic!("expected Elem, got {other:?}"),
    }
}

#[test]
fn element_frame_unknown_source() {
    let r = Element::try_from(
        b"java.base@17.0.17/java.io.BufferedReader.readLine(Unknown Source)".as_slice(),
    )
    .unwrap();
    match r {
        Element::Elem { method: frame, source } => {
            assert_eq!(frame, b"java.base@17.0.17/java.io.BufferedReader.readLine");
            assert!(matches!(source, Source::UnknownSource));
        }
        other => panic!("expected Elem, got {other:?}"),
    }
}

#[test]
fn element_frame_filename() {
    // BUG-1
    let r = Element::try_from(
        b"com.foo.WindowsSystemMonitor.getSystemMetrics(WindowsSystemMonitor.java:40)"
            .as_slice(),
    )
    .unwrap();
    match r {
        Element::Elem { method: frame, source } => {
            assert_eq!(frame, b"com.foo.WindowsSystemMonitor.getSystemMetrics");
            match source {
                Source::Filename { file, line } => {
                    assert_eq!(file, b"WindowsSystemMonitor.java");
                    assert_eq!(line, 40);
                }
                other => panic!("expected Filename, got {other:?}"),
            }
        }
        other => panic!("expected Elem, got {other:?}"),
    }
}

#[test]
fn element_frame_module_prefixed() {
    let r =
        Element::try_from(b"java.base@17.0.17/jdk.internal.misc.Unsafe.park(Native Method)".as_slice())
            .unwrap();
    match r {
        Element::Elem { method: frame, .. } => {
            assert_eq!(frame, b"java.base@17.0.17/jdk.internal.misc.Unsafe.park");
        }
        other => panic!("expected Elem, got {other:?}"),
    }
}

#[test]
fn element_frame_lambda_unknown_source() {
    let r = Element::try_from(
        b"com.adventnet.mfw.bean.BeanProxy$$Lambda$395/0x000001cf92566e40.call(Unknown Source)"
            .as_slice(),
    )
    .unwrap();
    match r {
        Element::Elem { method: frame, source } => {
            assert_eq!(
                frame,
                b"com.adventnet.mfw.bean.BeanProxy$$Lambda$395/0x000001cf92566e40.call"
            );
            assert!(matches!(source, Source::UnknownSource));
        }
        other => panic!("expected Elem, got {other:?}"),
    }
}

#[test]
fn element_frame_jdk_proxy() {
    let r = Element::try_from(b"jdk.proxy3/jdk.proxy3.$Proxy45.get(Unknown Source)".as_slice())
        .unwrap();
    match r {
        Element::Elem { method: frame, source } => {
            assert_eq!(frame, b"jdk.proxy3/jdk.proxy3.$Proxy45.get");
            assert!(matches!(source, Source::UnknownSource));
        }
        other => panic!("expected Elem, got {other:?}"),
    }
}

#[test]
fn element_frame_generated_method_accessor() {
    let r = Element::try_from(
        b"jdk.internal.reflect.GeneratedMethodAccessor112.invoke(Unknown Source)".as_slice(),
    )
    .unwrap();
    match r {
        Element::Elem { method: frame, source } => {
            assert_eq!(
                frame,
                b"jdk.internal.reflect.GeneratedMethodAccessor112.invoke"
            );
            assert!(matches!(source, Source::UnknownSource));
        }
        other => panic!("expected Elem, got {other:?}"),
    }
}

#[test]
fn element_frame_leading_whitespace_stripped() {
    let r = Element::try_from(b"  \t com.foo.Bar.baz(Native Method)".as_slice()).unwrap();
    match r {
        Element::Elem { method: frame, .. } => assert_eq!(frame, b"com.foo.Bar.baz"),
        other => panic!("expected Elem, got {other:?}"),
    }
}

#[test]
fn element_frame_nested_parens_takes_outer_open_inner_close() {
    // `take_until_inclusive(b"(")` finds FIRST `(`, then `take_within` finds first `)`.
    // So inputs with nested parens are parsed greedily-then-eagerly.
    let r = Element::try_from(b"a(b(c)d)".as_slice()).unwrap();
    match r {
        Element::Elem { method: frame, source } => {
            assert_eq!(frame, b"a");
            // source bytes between FIRST `(` and FIRST `)` = "b(c"
            match source {
                Source::Generated(inner) => assert_eq!(inner, b"b(c"),
                other => panic!("expected Generated, got {other:?}"),
            }
        }
        other => panic!("expected Elem, got {other:?}"),
    }
}

#[test]
fn element_frame_multiple_open_parens() {
    // "a((Native Method)" — frame = "a", source bytes between first `(` and first `)`.
    let r = Element::try_from(b"a((Native Method)".as_slice()).unwrap();
    match r {
        Element::Elem { method: frame, source } => {
            assert_eq!(frame, b"a");
            // first `)` closes at "Native Method", source = b"(Native Method"
            match source {
                Source::Generated(inner) => assert_eq!(inner, b"(Native Method"),
                other => panic!("expected Generated, got {other:?}"),
            }
        }
        other => panic!("expected Elem, got {other:?}"),
    }
}

#[test]
fn element_frame_empty_source_bytes_is_generated_empty() {
    let r = Element::try_from(b"()".as_slice()).unwrap();
    match r {
        Element::Elem { method: frame, source } => {
            assert_eq!(frame, b"");
            assert!(matches!(source, Source::Generated(b"")));
        }
        other => panic!("expected Elem, got {other:?}"),
    }
}

#[test]
fn element_frame_only_close_paren_errors() {
    // No `(` -> ParenNotFound
    assert!(matches!(
        Element::try_from(b")".as_slice()),
        Err(Parse::ParenNotFound)
    ));
}

#[test]
fn element_frame_only_open_paren_errors() {
    // `(` is found, frame = b"", then no `)` -> ParenNotFound
    assert!(matches!(
        Element::try_from(b"(".as_slice()),
        Err(Parse::ParenNotFound)
    ));
}

#[test]
fn element_frame_no_parens_errors() {
    assert!(matches!(
        Element::try_from(b"some.function.without.parens".as_slice()),
        Err(Parse::ParenNotFound)
    ));
}

#[test]
fn element_frame_missing_close_paren_errors() {
    assert!(matches!(
        Element::try_from(b"some.method(Native Method".as_slice()),
        Err(Parse::ParenNotFound)
    ));
}

#[test]
fn element_frame_empty_input_errors() {
    assert!(matches!(
        Element::try_from(b"".as_slice()),
        Err(Parse::ParenNotFound)
    ));
}

#[test]
fn element_frame_only_whitespace_errors() {
    assert!(matches!(
        Element::try_from(b"   \t   ".as_slice()),
        Err(Parse::ParenNotFound)
    ));
}

#[test]
fn element_frame_with_at_in_method_name_keeps_full_method() {
    // Method name itself can contain `@` (module syntax). The `(` is the real boundary.
    let r = Element::try_from(b"a@b@c(Native Method)".as_slice()).unwrap();
    match r {
        Element::Elem { method: frame, .. } => assert_eq!(frame, b"a@b@c"),
        other => panic!("expected Elem, got {other:?}"),
    }
}

#[test]
fn element_frame_source_with_trailing_bytes_after_close_paren_discarded() {
    // `take_within` consumes through the `)` — trailing bytes are silently lost.
    let r = Element::try_from(b"foo(Native Method) trailing garbage".as_slice()).unwrap();
    assert!(matches!(
        r,
        Element::Elem {
            source: Source::NativeMethod,
            ..
        }
    ));
}

// ============================================================================
// 5. Trace::try_from
// ============================================================================

#[test]
fn trace_basic_two_frames() {
    let input = b"\
java.lang.Throwable
\tat java.base@17.0.17/jdk.internal.misc.Unsafe.park(Native Method)
\tat java.base@17.0.17/java.util.concurrent.locks.LockSupport.parkNanos(Unknown Source)
"
    .as_slice();
    let trace = Trace::try_from(input).unwrap();
    assert_eq!(trace.0.len(), 2);
}

#[test]
fn trace_single_frame_native() {
    let input = b"java.lang.Throwable\n\tat foo.bar.Baz.qux(Native Method)\n".as_slice();
    let trace = Trace::try_from(input).unwrap();
    assert_eq!(trace.0.len(), 1);
}

#[test]
fn trace_throwable_only_with_newline() {
    let trace = Trace::try_from(b"java.lang.Throwable\n".as_slice()).unwrap();
    assert!(trace.0.is_empty());
}

#[test]
fn trace_throwable_only_no_newline() {
    let trace = Trace::try_from(b"java.lang.Throwable".as_slice()).unwrap();
    assert!(trace.0.is_empty());
}

#[test]
fn trace_throwable_with_trailing_whitespace_only() {
    let trace = Trace::try_from(b"java.lang.Throwable\n   \n  \t  ".as_slice()).unwrap();
    assert!(trace.0.is_empty());
}

#[test]
fn trace_missing_throwable_at_only() {
    assert!(matches!(
        Trace::try_from(b"\tat foo.bar.Baz.qux(Native Method)\n".as_slice()),
        Err(Parse::ThrowableNotFound)
    ));
}

#[test]
fn trace_missing_throwable_typo() {
    assert!(matches!(
        Trace::try_from(b"java.lang.Throwabl\n".as_slice()),
        Err(Parse::ThrowableNotFound)
    ));
}

#[test]
fn trace_missing_throwable_wrong_case() {
    assert!(matches!(
        Trace::try_from(b"Java.lang.Throwable\n".as_slice()),
        Err(Parse::ThrowableNotFound)
    ));
}

#[test]
fn trace_empty_input() {
    assert!(matches!(
        Trace::try_from(b"".as_slice()),
        Err(Parse::ThrowableNotFound)
    ));
}

#[test]
fn trace_whitespace_only_input() {
    assert!(matches!(
        Trace::try_from(b"   \n\t   ".as_slice()),
        Err(Parse::ThrowableNotFound)
    ));
}

#[test]
fn trace_at_not_found_garbage_line() {
    assert!(matches!(
        Trace::try_from(b"java.lang.Throwable\nrandom garbage line\n".as_slice()),
        Err(Parse::AtNotFound)
    ));
}

#[test]
fn trace_at_not_found_canonical_java_header_with_message_bug_2() {
    // BUG-2: real Java emits "java.lang.Throwable: some message" — the
    // canonical header — which this parser rejects.
    let input = b"\
java.lang.Throwable: connection refused
\tat foo.Bar.baz(Native Method)
"
    .as_slice();
    assert!(matches!(Trace::try_from(input), Err(Parse::AtNotFound)));
}

#[test]
fn trace_at_not_found_after_blank_lines() {
    let input = b"java.lang.Throwable\n   \n   \n  not_at_line\n".as_slice();
    assert!(matches!(Trace::try_from(input), Err(Parse::AtNotFound)));
}

#[test]
fn trace_at_not_found_lock_line_bug_3() {
    // BUG-3: lock lines mid-trace would error because `expect(b"at")` runs first.
    let input = b"\
java.lang.Throwable
\tat foo.Bar.acquire(Native Method)
\t- locked java.lang.Object@deadbeef
\tat foo.Bar.release(Native Method)
"
    .as_slice();
    assert!(matches!(Trace::try_from(input), Err(Parse::AtNotFound)));
}

#[test]
fn trace_propagates_paren_error_from_element() {
    let input = b"java.lang.Throwable\n\tat broken_no_parens\n".as_slice();
    assert!(matches!(Trace::try_from(input), Err(Parse::ParenNotFound)));
}

#[test]
fn trace_with_spaces_instead_of_tabs() {
    let input = b"\
java.lang.Throwable
    at jdk.internal.misc.Unsafe.park(Native Method)
    at java.util.concurrent.locks.LockSupport.parkNanos(Unknown Source)
"
    .as_slice();
    let trace = Trace::try_from(input).unwrap();
    assert_eq!(trace.0.len(), 2);
}

#[test]
fn trace_with_crlf_line_endings() {
    let input = b"java.lang.Throwable\r\n\tat foo.Bar.baz(Native Method)\r\n".as_slice();
    let trace = Trace::try_from(input).unwrap();
    assert_eq!(trace.0.len(), 1);
}

#[test]
fn trace_no_trailing_newline_drops_last_frame() {
    let input = b"\
java.lang.Throwable
\tat foo.Bar.first(Native Method)
\tat foo.Bar.second(Unknown Source)"
        .as_slice();
    let trace = Trace::try_from(input).unwrap();
    assert_eq!(trace.0.len(), 1, "last frame without `\\n` is silently dropped");
}

#[test]
fn trace_only_blank_lines_after_throwable() {
    let trace = Trace::try_from(b"java.lang.Throwable\n\n\n\n".as_slice()).unwrap();
    assert!(trace.0.is_empty());
}

#[test]
fn trace_deep_thirty_native_frames() {
    let mut input: Vec<u8> = Vec::from(b"java.lang.Throwable\n".as_slice());
    for i in 0..30 {
        input.extend_from_slice(
            format!("\tat com.example.Class{i}.method{i}(Native Method)\n").as_bytes(),
        );
    }
    let trace = Trace::try_from(input.as_slice()).unwrap();
    assert_eq!(trace.0.len(), 30);
    match &trace.0[0] {
        Element::Elem { method: frame, .. } => assert_eq!(*frame, b"com.example.Class0.method0"),
        _ => panic!(),
    }
    match &trace.0[29] {
        Element::Elem { method: frame, .. } => assert_eq!(*frame, b"com.example.Class29.method29"),
        _ => panic!(),
    }
}

#[test]
fn trace_pathologically_deep_one_thousand_frames() {
    let mut input: Vec<u8> = Vec::from(b"java.lang.Throwable\n".as_slice());
    for i in 0..1000 {
        input.extend_from_slice(format!("\tat c.f.M{i}(Native Method)\n").as_bytes());
    }
    let trace = Trace::try_from(input.as_slice()).unwrap();
    assert_eq!(trace.0.len(), 1000);
}

#[test]
fn trace_real_world_glowroot_pattern() {
    let input = b"\
java.lang.Throwable
\tat java.base@17.0.17/jdk.internal.misc.Unsafe.park(Native Method)
\tat java.base@17.0.17/java.util.concurrent.locks.LockSupport.parkNanos(Unknown Source)
\tat java.base@17.0.17/java.util.concurrent.locks.AbstractQueuedSynchronizer.acquire(Unknown Source)
\tat java.base@17.0.17/java.util.concurrent.locks.AbstractQueuedSynchronizer.tryAcquireSharedNanos(Unknown Source)
\tat java.base@17.0.17/java.util.concurrent.Semaphore.tryAcquire(Unknown Source)
"
    .as_slice();
    let trace = Trace::try_from(input).unwrap();
    assert_eq!(trace.0.len(), 5);
    assert!(matches!(
        &trace.0[0],
        Element::Elem {
            source: Source::NativeMethod,
            ..
        }
    ));
    for e in &trace.0[1..] {
        assert!(matches!(
            e,
            Element::Elem {
                source: Source::UnknownSource,
                ..
            }
        ));
    }
}

#[test]
fn trace_each_frame_method_name_borrowed_from_input() {
    let input: &[u8] = b"\
java.lang.Throwable
\tat foo.Bar.first(Native Method)
\tat foo.Bar.second(Unknown Source)
";
    let trace = Trace::try_from(input).unwrap();
    let lo = input.as_ptr() as usize;
    let hi = lo + input.len();
    for e in &trace.0 {
        if let Element::Elem { method: frame, .. } = e {
            let p = frame.as_ptr() as usize;
            assert!(p >= lo && p < hi, "frame must borrow from input");
        }
    }
}

// ============================================================================
// 6. Lifetimes / zero-copy
// ============================================================================

#[test]
fn lifetimes_object_class_borrows_input() {
    let input: &[u8] = b"java.lang.Object@deadbeef";
    let o = Object::try_from(input).unwrap();
    let lo = input.as_ptr() as usize;
    let hi = lo + input.len();
    let p = o.class.as_ptr() as usize;
    assert!(p >= lo && p < hi);
}

#[test]
fn lifetimes_source_generated_borrows_input() {
    let input: &[u8] = b"Lambda$1";
    let s = Source::try_from(input).unwrap();
    match s {
        Source::Generated(inner) => {
            assert_eq!(inner.as_ptr(), input.as_ptr());
        }
        _ => panic!(),
    }
}

#[test]
fn lifetimes_element_lock_object_borrows_input() {
    let input: &[u8] = b"- locked java.lang.Object@1";
    let e = Element::try_from(input).unwrap();
    let lo = input.as_ptr() as usize;
    let hi = lo + input.len();
    match e {
        Element::Lock(obj) => {
            let p = obj.class.as_ptr() as usize;
            assert!(p >= lo && p < hi);
        }
        _ => panic!(),
    }
}

// ============================================================================
// 7. Smoke / fuzz-ish — never panic on adversarial inputs
//
// These don't assert on result variants — just that the parser terminates
// without panicking on bytes that an attacker (or a corrupt log) might feed it.
// ============================================================================

#[test]
fn smoke_source_all_byte_values() {
    // Every possible single-byte input.
    for b in 0u8..=255 {
        let _ = Source::try_from([b].as_slice());
    }
}

#[test]
fn smoke_object_random_bytes_no_panic() {
    let inputs: &[&[u8]] = &[
        b"@",
        b"@@",
        b"@@@",
        b"x@y@z",
        b"\0@\0",
        b"\xff@\xfe",
        b"\n@\n",
        b"   @   ",
        &[0xff; 64],
        &[b'@'; 64],
    ];
    for input in inputs {
        let _ = Object::try_from(*input);
    }
}

#[test]
fn smoke_element_random_bytes_no_panic() {
    let inputs: &[&[u8]] = &[
        b"",
        b"(",
        b")",
        b"()",
        b"))((",
        b"-",
        b"- ",
        b"- l",
        b"- locked",
        b"- locked ",
        b"- locked @",
        b"(((((((",
        b")))))))",
        b"\0\0\0\0",
        b"\n\n\n\n",
        &[0xff; 128],
    ];
    for input in inputs {
        let _ = Element::try_from(*input);
    }
}

#[test]
fn smoke_trace_random_bytes_no_panic() {
    let inputs: &[&[u8]] = &[
        b"",
        b"\0",
        b"java.lang.Throwable",
        b"java.lang.Throwable\n",
        b"java.lang.Throwable\n\tat",
        b"java.lang.Throwable\n\tat\n",
        b"java.lang.Throwable\n\tat (",
        b"java.lang.Throwable\n\tat )",
        b"java.lang.Throwable\n\tat ()",
        b"java.lang.Throwable: msg\n\tat foo(Native Method)\n",
        &[0xff; 256],
    ];
    for input in inputs {
        let _ = Trace::try_from(*input);
    }
}

#[test]
fn smoke_trace_thousand_garbage_lines_after_at_terminates() {
    // The driver should error fast on the very first non-`at` line.
    let mut input: Vec<u8> = Vec::from(b"java.lang.Throwable\n".as_slice());
    for _ in 0..1000 {
        input.extend_from_slice(b"garbage line\n");
    }
    assert!(matches!(
        Trace::try_from(input.as_slice()),
        Err(Parse::AtNotFound)
    ));
}
