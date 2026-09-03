//! Tests for `TimeSpan` formatting: `Display`, `to_string_format`, and `try_format`.

use cs_timespan_automated_v1::{TimeSpan, TimeSpanError};

/// `Display` mirrors C#'s parameterless `ToString()`, which delegates to
/// `TimeSpanFormat.FormatC` — the invariant, culture-independent constant "c" format
/// `[-][d.]hh:mm:ss[.fffffff]`. Only the constant-format rows of the C# test's
/// `ToString_TestData` are ported here (the `null`/`"c"`/`"t"`/`"T"` format-string rows,
/// which C# routes to the same `FormatC` path) — the `"g"`/`"G"` general-format rows and
/// the culture-aware overloads are out of scope; see the `Display` impl's doc comment.
///
/// Cf. TimeSpanTests.cs#L1539-L1591 (`ToString_TestData`, constant-format rows),
/// TimeSpanTests.cs#L1656-L1666 (`ToString_Valid`)
#[test]
fn display_constant_format() {
    assert_eq!(
        "142.21:21:18.9101112",
        TimeSpan::from_ticks(123_456_789_101_112).to_string()
    );
    assert_eq!("00:00:00", TimeSpan::ZERO.to_string());
    assert_eq!("00:00:00.0000001", TimeSpan::from_ticks(1).to_string());
    assert_eq!("-00:00:00.0000001", TimeSpan::from_ticks(-1).to_string());
    assert_eq!("10675199.02:48:05.4775807", TimeSpan::MAX.to_string());
    assert_eq!("-10675199.02:48:05.4775808", TimeSpan::MIN.to_string());
    assert_eq!(
        "01:02:03",
        TimeSpan::builder()
            .hours(1)
            .minutes(2)
            .seconds(3)
            .build()
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "-01:02:03",
        (-TimeSpan::builder()
            .hours(1)
            .minutes(2)
            .seconds(3)
            .build()
            .unwrap())
        .to_string()
    );
    assert_eq!(
        "12:34:56",
        TimeSpan::builder()
            .hours(12)
            .minutes(34)
            .seconds(56)
            .build()
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "13.10:56:23",
        TimeSpan::builder()
            .days(12)
            .hours(34)
            .minutes(56)
            .seconds(23)
            .build()
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "13.10:56:23.0450000",
        TimeSpan::builder()
            .days(12)
            .hours(34)
            .minutes(56)
            .seconds(23)
            .milliseconds(45)
            .build()
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "23:59:59.9990000",
        TimeSpan::builder()
            .hours(23)
            .minutes(59)
            .seconds(59)
            .milliseconds(999)
            .build()
            .unwrap()
            .to_string()
    );
}

/// `to_string_format` mirrors C#'s `ToString(string? format)` for the constant `"c"`
/// format and its `"t"`/`"T"` aliases, plus an empty/absent format string — all four
/// route to the same invariant output as `Display`. Only the invariant-culture rows of
/// the C# test's `ToString_TestData` constant-format block are ported (culture is
/// always ignored for `"c"`/`"t"`/`"T"`, matching `Display`'s existing scope).
///
/// Cf. TimeSpanTests.cs#L1572-L1591 (`ToString_TestData`, constant-format rows),
/// TimeSpanTests.cs#L1656-L1669 (`ToString_Valid`)
#[test]
fn to_string_format_constant() {
    let cases: &[(TimeSpan, &str)] = &[
        (
            TimeSpan::from_ticks(123_456_789_101_112),
            "142.21:21:18.9101112",
        ),
        (TimeSpan::ZERO, "00:00:00"),
        (TimeSpan::from_ticks(1), "00:00:00.0000001"),
        (TimeSpan::from_ticks(-1), "-00:00:00.0000001"),
        (TimeSpan::MAX, "10675199.02:48:05.4775807"),
        (TimeSpan::MIN, "-10675199.02:48:05.4775808"),
    ];

    for (input, expected) in cases {
        for format in ["", "c", "t", "T"] {
            assert_eq!(Ok((*expected).to_string()), input.to_string_format(format));
        }
    }
}

/// The general short `"g"` format: variable-width hours (one digit when `< 10`), the
/// day segment omitted entirely when zero, and a fraction shown only when non-zero
/// with trailing zeros trimmed. Only the invariant-culture rows are ported — the
/// `NumberDecimalSeparator`-varies-by-culture rows are permanently out of scope (this
/// crate has no culture/locale support anywhere, matching `Display`'s existing
/// invariant-only scope for `"c"`).
///
/// Cf. TimeSpanFormat.cs `TryFormatStandard` (`StandardFormat.g` branch),
/// TimeSpanTests.cs#L1593-L1606 (`ToString_TestData`, general short format rows)
#[test]
fn to_string_format_general_short() {
    assert_eq!(
        Ok("142:21:21:18.9101112".to_string()),
        TimeSpan::from_ticks(123_456_789_101_112).to_string_format("g")
    );
    assert_eq!(
        Ok("0:00:00".to_string()),
        TimeSpan::ZERO.to_string_format("g")
    );
    assert_eq!(
        Ok("0:00:00.0000001".to_string()),
        TimeSpan::from_ticks(1).to_string_format("g")
    );
    assert_eq!(
        Ok("-0:00:00.0000001".to_string()),
        TimeSpan::from_ticks(-1).to_string_format("g")
    );
    assert_eq!(
        Ok("10675199:2:48:05.4775807".to_string()),
        TimeSpan::MAX.to_string_format("g")
    );
    assert_eq!(
        Ok("-10675199:2:48:05.4775808".to_string()),
        TimeSpan::MIN.to_string_format("g")
    );
    assert_eq!(
        Ok("1:02:03".to_string()),
        TimeSpan::builder()
            .hours(1)
            .minutes(2)
            .seconds(3)
            .build()
            .unwrap()
            .to_string_format("g")
    );
    assert_eq!(
        Ok("-1:02:03".to_string()),
        (-TimeSpan::builder()
            .hours(1)
            .minutes(2)
            .seconds(3)
            .build()
            .unwrap())
        .to_string_format("g")
    );
    assert_eq!(
        Ok("12:34:56".to_string()),
        TimeSpan::builder()
            .hours(12)
            .minutes(34)
            .seconds(56)
            .build()
            .unwrap()
            .to_string_format("g")
    );
    assert_eq!(
        Ok("13:10:56:23".to_string()),
        TimeSpan::builder()
            .days(12)
            .hours(34)
            .minutes(56)
            .seconds(23)
            .build()
            .unwrap()
            .to_string_format("g")
    );
    assert_eq!(
        Ok("13:10:56:23.045".to_string()),
        TimeSpan::builder()
            .days(12)
            .hours(34)
            .minutes(56)
            .seconds(23)
            .milliseconds(45)
            .build()
            .unwrap()
            .to_string_format("g")
    );
    assert_eq!(
        Ok("23:59:59.999".to_string()),
        TimeSpan::builder()
            .hours(23)
            .minutes(59)
            .seconds(59)
            .milliseconds(999)
            .build()
            .unwrap()
            .to_string_format("g")
    );
}

/// The general long `"G"` format: always two-digit hours, the day segment always
/// present (`"0:"` when zero), and the fraction always shown at full 7-digit width.
/// Only the invariant-culture rows are ported; see `to_string_format_general_short`'s
/// doc comment for why the culture-varying rows are excluded.
///
/// Cf. TimeSpanFormat.cs `TryFormatStandard` (`StandardFormat.G` branch),
/// TimeSpanTests.cs#L1624-L1636 (`ToString_TestData`, general long format rows)
#[test]
fn to_string_format_general_long() {
    assert_eq!(
        Ok("142:21:21:18.9101112".to_string()),
        TimeSpan::from_ticks(123_456_789_101_112).to_string_format("G")
    );
    assert_eq!(
        Ok("0:00:00:00.0000000".to_string()),
        TimeSpan::ZERO.to_string_format("G")
    );
    assert_eq!(
        Ok("0:00:00:00.0000001".to_string()),
        TimeSpan::from_ticks(1).to_string_format("G")
    );
    assert_eq!(
        Ok("-0:00:00:00.0000001".to_string()),
        TimeSpan::from_ticks(-1).to_string_format("G")
    );
    assert_eq!(
        Ok("10675199:02:48:05.4775807".to_string()),
        TimeSpan::MAX.to_string_format("G")
    );
    assert_eq!(
        Ok("-10675199:02:48:05.4775808".to_string()),
        TimeSpan::MIN.to_string_format("G")
    );
    assert_eq!(
        Ok("0:01:02:03.0000000".to_string()),
        TimeSpan::builder()
            .hours(1)
            .minutes(2)
            .seconds(3)
            .build()
            .unwrap()
            .to_string_format("G")
    );
    assert_eq!(
        Ok("-0:01:02:03.0000000".to_string()),
        (-TimeSpan::builder()
            .hours(1)
            .minutes(2)
            .seconds(3)
            .build()
            .unwrap())
        .to_string_format("G")
    );
    assert_eq!(
        Ok("0:12:34:56.0000000".to_string()),
        TimeSpan::builder()
            .hours(12)
            .minutes(34)
            .seconds(56)
            .build()
            .unwrap()
            .to_string_format("G")
    );
    assert_eq!(
        Ok("13:10:56:23.0000000".to_string()),
        TimeSpan::builder()
            .days(12)
            .hours(34)
            .minutes(56)
            .seconds(23)
            .build()
            .unwrap()
            .to_string_format("G")
    );
    assert_eq!(
        Ok("13:10:56:23.0450000".to_string()),
        TimeSpan::builder()
            .days(12)
            .hours(34)
            .minutes(56)
            .seconds(23)
            .milliseconds(45)
            .build()
            .unwrap()
            .to_string_format("G")
    );
    assert_eq!(
        Ok("0:23:59:59.9990000".to_string()),
        TimeSpan::builder()
            .hours(23)
            .minutes(59)
            .seconds(59)
            .milliseconds(999)
            .build()
            .unwrap()
            .to_string_format("G")
    );
}

/// The custom-format-string mini-language (`TimeSpanFormat.FormatCustomized`):
/// `%d`/`dd`...`dddddddd` (day), `%h`/`hh` (hour), `%m`/`mm` (minute), `%s`/`ss`
/// (second), `%f`/`ff`...`fffffff` (fraction, truncated, always shown), `%F`/`FF`...
/// `FFFFFFF` (fraction, trailing zeros dropped, omitted if empty), and `\`-escaped
/// literal text.
///
/// Cf. TimeSpanFormat.cs#L296-455 (`FormatCustomized`), TimeSpanTests.cs#L1545-1570
/// (`ToString_TestData`, custom-format rows)
#[test]
fn to_string_format_custom() {
    let input = TimeSpan::from_ticks(123_456_789_101_112);

    let cases: &[(&str, &str)] = &[
        ("%d", "142"),
        ("dd", "142"),
        ("%h", "21"),
        ("hh", "21"),
        ("%m", "21"),
        ("mm", "21"),
        ("%s", "18"),
        ("ss", "18"),
        ("%f", "9"),
        ("ff", "91"),
        ("fff", "910"),
        ("ffff", "9101"),
        ("fffff", "91011"),
        ("ffffff", "910111"),
        ("fffffff", "9101112"),
        ("%F", "9"),
        ("FF", "91"),
        ("FFF", "91"),
        ("FFFF", "9101"),
        ("FFFFF", "91011"),
        ("FFFFFF", "910111"),
        ("FFFFFFF", "9101112"),
        ("dd\\.ss", "142.18"),
        ("dddddd\\.ss", "000142.18"),
    ];

    for (format, expected) in cases {
        assert_eq!(
            Ok((*expected).to_string()),
            input.to_string_format(format),
            "format {format:?}"
        );
    }
}

/// Quoted literal spans (`'...'`/`"..."`) are copied verbatim into the output,
/// including `\`-escaped characters within the quotes.
///
/// Cf. TimeSpanFormat.cs#L405-408 (`FormatCustomized`'s `'\''`/`'"'` case),
/// DateTimeFormat.cs#L284-337 (`ParseQuoteString`)
#[test]
fn to_string_format_custom_quoted_literal() {
    let ts = TimeSpan::builder()
        .hours(1)
        .minutes(2)
        .seconds(3)
        .build()
        .unwrap();
    assert_eq!(
        Ok("hh is 01".to_string()),
        ts.to_string_format("'hh is 'hh")
    );
    assert_eq!(
        Ok("hh is 01".to_string()),
        ts.to_string_format("\"hh is \"hh")
    );
    // A backslash-escaped character inside a quoted span is unescaped into the
    // literal output, per `DateTimeFormat.ParseQuoteString`'s own `\`-handling.
    assert_eq!(
        Ok("it's 01".to_string()),
        ts.to_string_format("'it\\'s 'hh")
    );
}

/// `FormatCustomized` never writes a sign character itself — unlike the standard
/// `"c"`/`"g"`/`"G"` formats (which all prepend `-` for a negative `TimeSpan`), a
/// custom format string has no specifier for the sign, so a negative `TimeSpan`
/// formats identically to its positive magnitude. This is a genuine upstream quirk
/// (no case in `FormatCustomized`'s switch ever emits `-`), not a bug this port
/// introduces.
///
/// Cf. TimeSpanFormat.cs#L301-312 (`day`/`time` are negated to non-negative
/// magnitudes before the tokenizer loop runs; no `-` is ever appended)
#[test]
fn to_string_format_custom_no_sign() {
    let positive = TimeSpan::builder()
        .hours(1)
        .minutes(2)
        .seconds(3)
        .build()
        .unwrap();
    let negative = -positive;
    assert_eq!(
        Ok("01:02:03".to_string()),
        positive.to_string_format("hh\\:mm\\:ss")
    );
    assert_eq!(
        Ok("01:02:03".to_string()),
        negative.to_string_format("hh\\:mm\\:ss")
    );
}

/// A single-character format outside `"c"`/`"t"`/`"T"`/`"g"`/`"G"` is always rejected
/// at the top level (C#'s `Format`/`TryFormat` special-case format strings of length 1
/// entirely separately from the custom-format tokenizer, even though some of those
/// single characters, e.g. `"d"`, would otherwise be valid custom-format tokens). A
/// syntactically-invalid custom format string (length != 1) reports
/// [`TimeSpanError::InvalidFormat`] rather than panicking, mirroring C#'s
/// `FormatException`.
///
/// Cf. TimeSpanFormat.cs#L26-41 (`Format`'s length-1 special case, checked before
/// ever reaching `FormatCustomized`), TimeSpanTests.cs#L1671-L1684
/// (`ToString_InvalidFormat_TestData`, `ToString_InvalidFormat_ThrowsFormatException`)
#[test]
fn to_string_format_invalid() {
    // TimeSpanTests.cs#L1673-L1676: single characters that aren't valid standard
    // format specifiers (uppercase "C" is deliberately invalid in C# too - only
    // lowercase "c" is the constant format; "F"/"d" are custom-format-only tokens,
    // never reachable as a length-1 format string).
    for format in ["y", "F", "C", "d"] {
        assert_eq!(
            Err(TimeSpanError::InvalidFormat),
            TimeSpan::ZERO.to_string_format(format)
        );
    }
    // TimeSpanTests.cs#L1674: "cc" is a 2-character custom format string in C# -
    // invalid there too, since 'c' isn't a recognized custom-format token.
    for format in [
        "cc",            // 'c' isn't a recognized custom-format token
        "hhh",           // 'h' run > 2 (TimeSpanFormat.cs#L326-329)
        "mmm",           // 'm' run > 2 (TimeSpanFormat.cs#L334-337)
        "sss",           // 's' run > 2 (TimeSpanFormat.cs#L342-345)
        "ffffffff",      // 'f' run > 7 (TimeSpanFormat.cs#L353-356)
        "FFFFFFFF",      // 'F' run > 7 (TimeSpanFormat.cs#L367-370)
        "ddddddddd",     // 'd' run > 8 (TimeSpanFormat.cs#L398-401)
        "'unterminated", // missing closing quote (DateTimeFormat.cs#L327-331)
        "'bad\\",        // '\' at the end of a quoted span (DateTimeFormat.cs#L309-319)
        "dd%",           // trailing '%' (TimeSpanFormat.cs#L416-429)
        "dd%%",          // "%%" is disallowed (TimeSpanFormat.cs#L416-429)
        "dd\\",          // trailing '\' (TimeSpanFormat.cs#L436-447)
        "dXd",           // unquoted/unescaped literal character (TimeSpanFormat.cs#L449-451)
    ] {
        assert_eq!(
            Err(TimeSpanError::InvalidFormat),
            TimeSpan::ZERO.to_string_format(format),
            "format {format:?}"
        );
    }
}

/// The non-allocating counterpart to `to_string_format`: writes UTF-8 bytes directly
/// into a caller-provided buffer instead of allocating a `String`. Mirrors C#'s
/// `TryFormat(Span<char>, out int charsWritten, ...)`/`TryFormat(Span<byte>, out int
/// bytesWritten, ...)` buffer-sizing behavior — a buffer exactly one byte too short
/// reports [`TimeSpanError::InsufficientBuffer`] and writes nothing, a buffer exactly
/// long enough succeeds and is filled completely, and a buffer one byte larger than
/// needed succeeds while leaving the trailing byte untouched.
///
/// Cf. TimeSpanTests.cs#L1843-L1888 (`TryFormat_Valid`, `ToString_TestData` rows)
#[test]
fn try_format_valid() {
    let cases: &[(TimeSpan, &str, &str)] = &[
        (
            TimeSpan::builder()
                .hours(1)
                .minutes(2)
                .seconds(3)
                .build()
                .unwrap(),
            "c",
            "01:02:03",
        ),
        (TimeSpan::ZERO, "c", "00:00:00"),
        (
            TimeSpan::from_ticks(123_456_789_101_112),
            "c",
            "142.21:21:18.9101112",
        ),
        (TimeSpan::MIN, "c", "-10675199.02:48:05.4775808"),
        (
            TimeSpan::builder()
                .hours(1)
                .minutes(2)
                .seconds(3)
                .build()
                .unwrap(),
            "g",
            "1:02:03",
        ),
        (TimeSpan::ZERO, "g", "0:00:00"),
        (TimeSpan::MIN, "g", "-10675199:2:48:05.4775808"),
        (
            TimeSpan::builder()
                .hours(1)
                .minutes(2)
                .seconds(3)
                .build()
                .unwrap(),
            "G",
            "0:01:02:03.0000000",
        ),
        (TimeSpan::ZERO, "G", "0:00:00:00.0000000"),
        (TimeSpan::MIN, "G", "-10675199:02:48:05.4775808"),
    ];

    for (input, format, expected) in cases {
        let expected_len = expected.len();

        // One byte too short: fails, and nothing is written.
        let mut too_small = vec![0u8; expected_len - 1];
        assert_eq!(
            Err(TimeSpanError::InsufficientBuffer),
            input.try_format(&mut too_small, format)
        );

        // Exactly long enough: succeeds, buffer filled exactly.
        let mut exact = vec![0u8; expected_len];
        let written = input.try_format(&mut exact, format).unwrap();
        assert_eq!(expected_len, written);
        assert_eq!(*expected, std::str::from_utf8(&exact).unwrap());

        // One byte larger than needed: succeeds, trailing byte left untouched.
        let mut larger = vec![0u8; expected_len + 1];
        let written = input.try_format(&mut larger, format).unwrap();
        assert_eq!(expected_len, written);
        assert_eq!(*expected, std::str::from_utf8(&larger[..written]).unwrap());
        assert_eq!(0, larger[larger.len() - 1]);
    }
}

/// The custom-format-string counterpart to `try_format_valid`: `try_format`'s
/// non-allocating path also covers the custom-format-string mini-language, not just
/// the five standard formats — mirroring `to_string_format_custom`'s cases and
/// `try_format_valid`'s buffer-sizing contract (one byte short fails and writes
/// nothing, exact-size succeeds and fills completely, one byte larger succeeds and
/// leaves the trailing byte untouched).
///
/// Cf. TimeSpanTests.cs#L1843-L1888 (`TryFormat_Valid`, sharing `ToString_TestData`
/// with `ToString(string)` — TimeSpanTests.cs#L1546-1570's custom-format rows)
#[test]
fn try_format_custom() {
    let input = TimeSpan::from_ticks(123_456_789_101_112);

    let cases: &[(&str, &str)] = &[
        ("%d", "142"),
        ("dd", "142"),
        ("%h", "21"),
        ("hh", "21"),
        ("%m", "21"),
        ("mm", "21"),
        ("%s", "18"),
        ("ss", "18"),
        ("%f", "9"),
        ("fffffff", "9101112"),
        ("%F", "9"),
        ("FFFFFFF", "9101112"),
        ("dd\\.ss", "142.18"),
        ("dddddd\\.ss", "000142.18"),
    ];

    for (format, expected) in cases {
        let expected_len = expected.len();

        // One byte too short: fails, and nothing is written.
        let mut too_small = vec![0u8; expected_len - 1];
        assert_eq!(
            Err(TimeSpanError::InsufficientBuffer),
            input.try_format(&mut too_small, format),
            "format {format:?}"
        );

        // Exactly long enough: succeeds, buffer filled exactly.
        let mut exact = vec![0u8; expected_len];
        let written = input.try_format(&mut exact, format).unwrap();
        assert_eq!(expected_len, written, "format {format:?}");
        assert_eq!(
            *expected,
            std::str::from_utf8(&exact).unwrap(),
            "format {format:?}"
        );

        // One byte larger than needed: succeeds, trailing byte left untouched.
        let mut larger = vec![0u8; expected_len + 1];
        let written = input.try_format(&mut larger, format).unwrap();
        assert_eq!(expected_len, written, "format {format:?}");
        assert_eq!(
            *expected,
            std::str::from_utf8(&larger[..written]).unwrap(),
            "format {format:?}"
        );
        assert_eq!(0, larger[larger.len() - 1], "format {format:?}");
    }
}

/// Custom format strings never write a sign character, matching
/// `to_string_format_custom_no_sign` — verified here through the non-allocating
/// `try_format` path too.
///
/// Cf. TimeSpanFormat.cs#L301-312
#[test]
fn try_format_custom_no_sign() {
    let positive = TimeSpan::builder()
        .hours(1)
        .minutes(2)
        .seconds(3)
        .build()
        .unwrap();
    let negative = -positive;

    let mut buf = [0u8; 8];
    let written = positive.try_format(&mut buf, "hh\\:mm\\:ss").unwrap();
    assert_eq!("01:02:03", std::str::from_utf8(&buf[..written]).unwrap());

    let mut buf = [0u8; 8];
    let written = negative.try_format(&mut buf, "hh\\:mm\\:ss").unwrap();
    assert_eq!("01:02:03", std::str::from_utf8(&buf[..written]).unwrap());
}

/// Mirrors `to_string_format_invalid`, but through `try_format`: an invalid format
/// string reports [`TimeSpanError::InvalidFormat`] regardless of buffer size, checked
/// before any buffer-length validation (matching C#, where `FormatException` is thrown
/// even when passed a 1-element destination span) — including a syntactically invalid
/// *custom* format string, not just an invalid single-character standard format.
///
/// Cf. TimeSpanTests.cs#L1890-L1896 (`TryFormat_InvalidFormat_ThrowsFormatException`)
#[test]
fn try_format_invalid_format() {
    for format in ["y", "F", "C", "cc"] {
        let mut buf = [0u8; 1];
        assert_eq!(
            Err(TimeSpanError::InvalidFormat),
            TimeSpan::ZERO.try_format(&mut buf, format)
        );
    }

    // "hhh": an 'h' run > 2 is a syntactically invalid *custom* format string
    // (TimeSpanFormat.cs#L326-329) — rejected the same way as an invalid standard
    // format, even though it's a multi-character string that reaches the custom-
    // format tokenizer rather than the standard-format special case.
    let mut buf = [0u8; 1];
    assert_eq!(
        Err(TimeSpanError::InvalidFormat),
        TimeSpan::from_ticks(123_456_789_101_112).try_format(&mut buf, "hhh")
    );
}
