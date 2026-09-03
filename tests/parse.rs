//! Tests for `TimeSpan` parsing: `parse`, `parse_exact`, and `parse_exact_multiple`.

use cs_timespan_automated_v1::{TimeSpan, TimeSpanError, TimeSpanStyles};

/// Cf. TimeSpanTests.cs#L1006-L1104 (`Parse_Valid_TestData`/`Parse`), restricted to the
/// `provider: null`/`CultureInfo.InvariantCulture` rows — culture-specific rows (e.g. the
/// `hr-HR` comma-decimal case) are out of scope for invariant-only `from_str`.
///
/// Expected tick counts were cross-checked against a live `dotnet100csharpcoreclr` run of
/// `TimeSpan.Parse(input, CultureInfo.InvariantCulture).Ticks` for every row below, not just
/// hand-derived from the day/hour/minute/second/ms constants.
#[test]
fn parse_valid() {
    // (input, expected ticks, whether "-" + input is also expected to parse to the negation
    // — false only for the leading-whitespace rows, mirroring the C# test's
    // `!char.IsWhiteSpace(input[0])` guard on the negation assertion)
    let cases: [(&str, i64, bool); 38] = [
        ("       12:24:02", 446_420_000_000, false),
        ("12:24:02      ", 446_420_000_000, true),
        ("     12:24:02      ", 446_420_000_000, false),
        ("0", 0, true),
        ("12:24", 446_400_000_000, true),
        ("12:24:02", 446_420_000_000, true),
        ("12.03:04", 10_478_400_000_000, true),
        ("12:24:02.01", 446_420_100_000, true),
        ("1:1:1.0", 36_610_000_000, true),
        ("1:1:1.0000000", 36_610_000_000, true),
        ("1:1:1.1", 36_611_000_000, true),
        ("1:1:1.01", 36_610_100_000, true),
        ("1:1:1.001", 36_610_010_000, true),
        ("1:1:1.0001", 36_610_001_000, true),
        ("1:1:1.00001", 36_610_000_100, true),
        ("1:1:1.000001", 36_610_000_010, true),
        ("1:1:1.0000001", 36_610_000_001, true),
        ("1.12:24:02", 1_310_420_000_000, true),
        ("1:12:24:02", 1_310_420_000_000, true),
        ("01.23:45:.67", 1_719_006_700_000, true),
        ("1.12:24:02.999", 1_310_429_990_000, true),
        ("1:1:.1", 36_601_000_000, true),
        ("1:1:.01", 36_600_100_000, true),
        ("1:1:.001", 36_600_010_000, true),
        ("1:1:.0001", 36_600_001_000, true),
        ("1:1:.00001", 36_600_000_100, true),
        ("1:1:.000001", 36_600_000_010, true),
        ("1:1:.0000001", 36_600_000_001, true),
        ("10675199", 9_223_371_936_000_000_000, true),
        ("10675199:00:00", 9_223_371_936_000_000_000, true),
        ("10675199:02:00:00", 9_223_372_008_000_000_000, true),
        ("10675199:02:48:00", 9_223_372_036_800_000_000, true),
        ("10675199:02:48:05", 9_223_372_036_850_000_000, true),
        ("10675199:02:48:05.4775", 9_223_372_036_854_775_000, true),
        ("00:00:59", 590_000_000, true),
        ("00:59:00", 35_400_000_000, true),
        ("23:00:00", 828_000_000_000, true),
        ("24:00:00", 20_736_000_000_000, true),
    ];

    for (input, expected_ticks, negatable) in cases {
        let expected = TimeSpan::from_ticks(expected_ticks);
        assert_eq!(Ok(expected), input.parse::<TimeSpan>(), "parsing {input:?}");

        if negatable {
            let negated = format!("-{input}");
            assert_eq!(
                Ok(TimeSpan::from_ticks(-expected_ticks)),
                negated.parse::<TimeSpan>(),
                "parsing {negated:?}"
            );
        }
    }
}

/// Cf. TimeSpanTests.cs#L1106-L1160 (`Parse_Invalid_TestData`/`Parse_Invalid`), restricted to
/// the `provider: null` rows minus the `null` input case (no `&str` equivalent of a null
/// `string` to parse) and the `hr-HR` culture-specific row.
#[test]
fn parse_invalid() {
    let cases: [(&str, TimeSpanError); 24] = [
        ("", TimeSpanError::InvalidFormat),
        ("-", TimeSpanError::InvalidFormat),
        ("garbage", TimeSpanError::InvalidFormat),
        ("12/12/12", TimeSpanError::InvalidFormat),
        ("00:", TimeSpanError::InvalidFormat),
        ("00:00:-01", TimeSpanError::InvalidFormat),
        ("\u{0}12:34:56", TimeSpanError::InvalidFormat),
        ("1\u{0}2:34:56", TimeSpanError::InvalidFormat),
        ("12\u{0}:34:56", TimeSpanError::InvalidFormat),
        ("00:00::00", TimeSpanError::InvalidFormat),
        ("00:00:00:", TimeSpanError::InvalidFormat),
        ("00:00:00:00:00:00:00:00", TimeSpanError::InvalidFormat),
        ("1:1:1.99999999", TimeSpanError::Overflow),
        ("2147483647", TimeSpanError::Overflow),
        ("2147483648", TimeSpanError::Overflow),
        ("10675200", TimeSpanError::Overflow),
        ("10675200:00:00", TimeSpanError::Overflow),
        ("10675199:03:00:00", TimeSpanError::Overflow),
        ("10675199:02:49:00", TimeSpanError::Overflow),
        ("10675199:02:48:06", TimeSpanError::Overflow),
        ("-10675199:02:48:06", TimeSpanError::Overflow),
        ("10675199:02:48:05.4776", TimeSpanError::Overflow),
        ("-10675199:02:48:05.4776", TimeSpanError::Overflow),
        ("00:00:60", TimeSpanError::Overflow),
    ];

    for (input, expected_err) in cases {
        assert_eq!(
            Err(expected_err),
            input.parse::<TimeSpan>(),
            "parsing {input:?}"
        );
    }

    // "00:60:00" and "24:00" (overflowing minutes/hours) are also part of the upstream
    // data set but share a row type with the table above; kept separate only because
    // Rust doesn't need the array to be homogeneous in any special way — listed here for
    // strict 1:1 parity with the upstream rows rather than folding them silently into the
    // table above out of order.
    assert_eq!(
        Err(TimeSpanError::Overflow),
        "00:60:00".parse::<TimeSpan>(),
        "parsing \"00:60:00\""
    );
    assert_eq!(
        Err(TimeSpanError::Overflow),
        "24:00".parse::<TimeSpan>(),
        "parsing \"24:00\""
    );
}

/// Cf. TimeSpanTests.cs#L1730-L1752 (`ParseDifferentLengthFractionWithLeadingZerosData`/
/// `ParseDifferentLengthFractionWithLeadingZeros`), `Parse` half only — the `ParseExact(..,
/// "g", ..)` half is out of scope for invariant-only `from_str`.
#[test]
fn parse_different_length_fraction_with_leading_zeros() {
    let cases: [(&str, i64); 11] = [
        ("00:00:00.00000001", 0),
        ("00:00:00.00000005", 1),
        ("00:00:00.09999999", 1_000_000),
        ("00:00:00.0268435455", 268_435),
        ("00:00:00.01", 100_000),
        ("0:00:00.01000000", 100_000),
        ("0:00:00.010000000", 100_000),
        ("0:00:00.0123456", 123_456),
        ("0:00:00.00123456", 12_346),
        ("0:00:00.00000098", 10),
        ("0:00:00.00000099", 10),
    ];

    for (input, expected_ticks) in cases {
        assert_eq!(
            Ok(TimeSpan::from_ticks(expected_ticks)),
            input.parse::<TimeSpan>(),
            "parsing {input:?}"
        );
    }
}

/// Regression test for a `normalize_fraction` panic on pathological fraction tokens:
/// a fraction with enough leading zeros *and* enough significant digits pushes
/// `total_digits - MAX_FRACTION_DIGITS` past 7, indexing `POWERS_OF_TEN` (an 8-element
/// array, valid indices 0..=7) out of bounds. C#'s `Pow10UpToMaxFractionDigits` has the
/// identical unbounded index into its own 8-element `powersOfTen` span, guarded only by
/// a `Debug.Assert` that doesn't suppress the span's own (always-on) bounds check — so
/// upstream would throw `IndexOutOfRangeException` for this exact input too, rather than
/// the `FormatException`/`OverflowException` its documented `Parse` contract promises.
/// This crate's own contract (no panics on malformed input, always a `Result`) is the
/// reason to fix this independent of C# sharing the same latent defect.
#[test]
fn parse_pathological_fraction_does_not_panic() {
    // zeroes=6, value=268_435_455 (9 digits) => total_digits=15, so the naive
    // `POWERS_OF_TEN[total_digits - MAX_FRACTION_DIGITS]` lookup would need index 8,
    // one past the array's last valid index (7).
    //
    // Expected: the fraction "000000268435455" is 0.000000268435455 seconds, which
    // rounded to 7-digit tick precision (round(0.000000268435455 * 1e7) = round(2.68435455))
    // is 3 ticks. h=1,m=2,s=3 contributes 3_723_000ms * 10_000 ticks/ms = 37_230_000_000
    // ticks, plus the 3 fraction ticks.
    assert_eq!(
        Ok(TimeSpan::from_ticks(37_230_000_003)),
        "1:2:3.000000268435455".parse::<TimeSpan>()
    );
}

/// Cf. TimeSpanTests.cs#L1162-L1206 (`ParseExact_Valid_TestData`), restricted to the
/// "Custom timespan formats" rows (TimeSpanTests.cs#L1191-L1205) — the standard
/// single-letter-format rows ("c"/"t"/"T"/"g"/"G", TimeSpanTests.cs#L1164-L1189) are out of
/// scope for `TimeSpan::parse_exact`'s narrow custom-format-string-only slice; see its doc
/// comment.
#[test]
fn parse_exact_valid() {
    let cases: [(&str, &str, TimeSpan); 14] = [
        (
            "12.23:32:43",
            r"dd\.h\:m\:s",
            TimeSpan::from_ticks(11_215_630_000_000),
        ),
        (
            "012.23:32:43.893",
            r"ddd\.h\:m\:s\.fff",
            TimeSpan::from_ticks(11_215_638_930_000),
        ),
        (
            "12.05:02:03",
            r"d\.hh\:mm\:ss",
            TimeSpan::from_ticks(10_549_230_000_000),
        ),
        (
            "12:34 minutes",
            r"mm\:ss\ \m\i\n\u\t\e\s",
            TimeSpan::from_ticks(7_540_000_000),
        ),
        (
            "12:34 minutes",
            r#"mm\:ss\ "minutes""#,
            TimeSpan::from_ticks(7_540_000_000),
        ),
        (
            "12:34 minutes",
            r"mm\:ss\ 'minutes'",
            TimeSpan::from_ticks(7_540_000_000),
        ),
        ("678", "fff", TimeSpan::from_ticks(6_780_000)),
        ("678", "FFF", TimeSpan::from_ticks(6_780_000)),
        ("3", "%d", TimeSpan::from_ticks(2_592_000_000_000)),
        ("3", "%h", TimeSpan::from_ticks(108_000_000_000)),
        ("3", "%m", TimeSpan::from_ticks(1_800_000_000)),
        ("3", "%s", TimeSpan::from_ticks(30_000_000)),
        ("3", "%f", TimeSpan::from_ticks(3_000_000)),
        ("3", "%F", TimeSpan::from_ticks(3_000_000)),
    ];

    for (input, format, expected) in cases {
        assert_eq!(
            Ok(expected),
            TimeSpan::parse_exact(input, format, TimeSpanStyles::None),
            "parsing {input:?} against format {format:?}"
        );
    }
}

/// Cf. TimeSpanTests.cs#L1230-L1241 (`ParseExact`'s `TimeSpanStyles.AssumeNegative`
/// assertion — gated there on `format` not being one of the five standard single-letter
/// formats, so only exercised here against a sample of the custom-format rows from
/// `parse_exact_valid` above).
#[test]
fn parse_exact_assume_negative() {
    let cases: [(&str, &str, TimeSpan); 3] = [
        (
            "12.23:32:43",
            r"dd\.h\:m\:s",
            TimeSpan::from_ticks(11_215_630_000_000),
        ),
        ("3", "%h", TimeSpan::from_ticks(108_000_000_000)),
        ("678", "fff", TimeSpan::from_ticks(6_780_000)),
    ];

    for (input, format, expected) in cases {
        assert_eq!(
            Ok(-expected),
            TimeSpan::parse_exact(input, format, TimeSpanStyles::AssumeNegative),
            "parsing {input:?} against format {format:?} with AssumeNegative"
        );
    }
}

/// Cf. TimeSpanTests.cs#L1252-L1304 (`ParseExact_Invalid_TestData`), restricted to rows
/// usable without a `null` `string`/`string[]` (no `&str` equivalent) and to the
/// format-agnostic `""`/`"garbage"`-style rows plus the "Custom timespan formats" section
/// (TimeSpanTests.cs#L1275-L1303) — the standard single-letter-format rows
/// (TimeSpanTests.cs#L1261-L1274, `"c"`/`"g"`/`"G"`) are out of scope for
/// `TimeSpan::parse_exact`'s narrow custom-format-string-only slice: a 1-character `format`
/// unconditionally returns `InvalidFormat` here regardless of what the real C# algorithm
/// for that particular standard format would have done with the input, so those rows would
/// only coincidentally match (or not) rather than actually exercise this port's algorithm.
#[test]
fn parse_exact_invalid() {
    let cases: [(&str, &str, TimeSpanError); 28] = [
        ("00:00:00", "", TimeSpanError::InvalidFormat),
        ("12.5:2", "V", TimeSpanError::InvalidFormat),
        ("12.35:32:43", r"dd\.h\:m\:s", TimeSpanError::Overflow),
        ("12.5:2:3", r"d\.hh\:mm\:ss", TimeSpanError::InvalidFormat),
        ("12.5:2", r"d\.hh\:mm\:ss", TimeSpanError::InvalidFormat),
        ("678", "ffff", TimeSpanError::InvalidFormat),
        ("00000012", "FFFFFFFF", TimeSpanError::InvalidFormat),
        ("12:034:56", r"hh\mm\ss", TimeSpanError::InvalidFormat),
        ("12:34:056", r"hh\mm\ss", TimeSpanError::InvalidFormat),
        (
            "12:34 minutes",
            r#"mm\:ss\ "minutes"#,
            TimeSpanError::InvalidFormat,
        ),
        (
            "12:34 minutes",
            r"mm\:ss\ 'minutes",
            TimeSpanError::InvalidFormat,
        ),
        (
            "12:34 mints",
            r#"mm\:ss\ "minutes""#,
            TimeSpanError::InvalidFormat,
        ),
        (
            "12:34 mints",
            r"mm\:ss\ 'minutes'",
            TimeSpanError::InvalidFormat,
        ),
        ("1", "d%", TimeSpanError::InvalidFormat),
        ("1", "%%d", TimeSpanError::InvalidFormat),
        ("12:34:56", r"hhh\:mm\:ss", TimeSpanError::InvalidFormat),
        ("12:34:56", r"hh\:hh\:ss", TimeSpanError::InvalidFormat),
        ("123:34:56", r"hh\:mm\:ss", TimeSpanError::InvalidFormat),
        ("12:34:56", r"hh\:mmm\:ss", TimeSpanError::InvalidFormat),
        ("12:34:56", r"hh\:mm\:mm", TimeSpanError::InvalidFormat),
        ("12:345:56", r"hh\:mm\:ss", TimeSpanError::InvalidFormat),
        ("12:34:56", r"hh\:mm\:sss", TimeSpanError::InvalidFormat),
        ("12:34:56", r"hh\:ss\:ss", TimeSpanError::InvalidFormat),
        ("12:45", "ff:ff", TimeSpanError::InvalidFormat),
        ("000000123", "ddddddddd", TimeSpanError::InvalidFormat),
        ("12:34:56", "dd:dd:hh", TimeSpanError::InvalidFormat),
        ("123:45", "dd:hh", TimeSpanError::InvalidFormat),
        ("12:34", "dd:vv", TimeSpanError::InvalidFormat),
    ];

    for (input, format, expected_err) in cases {
        assert_eq!(
            Err(expected_err),
            TimeSpan::parse_exact(input, format, TimeSpanStyles::None),
            "parsing {input:?} against format {format:?}"
        );
    }
}

/// Cf. TimeSpanTests.cs#L1163-L1168 (`ParseExact_Valid_TestData`'s `"c"`/`"t"`/`"T"` rows —
/// all three characters dispatch to the exact same `TryParseTimeSpanConstant` algorithm
/// upstream, TimeSpanParse.cs#L1237-1239, hence looping over all three here too).
#[test]
fn parse_exact_standard_constant_valid() {
    let cases: [(&str, TimeSpan); 3] = [
        ("12:24:02", TimeSpan::from_ticks(446_420_000_000)),
        ("1.12:24:02", TimeSpan::from_ticks(1_310_420_000_000)),
        ("-01.07:45:16.999", -TimeSpan::from_ticks(1_143_169_990_000)),
    ];

    for format in ["c", "t", "T"] {
        for (input, expected) in cases {
            assert_eq!(
                Ok(expected),
                TimeSpan::parse_exact(input, format, TimeSpanStyles::None),
                "parsing {input:?} against format {format:?}"
            );
        }
    }
}

/// Cf. TimeSpanTests.cs#L1170-L1183 (`ParseExact_Valid_TestData`'s `"g"` rows).
#[test]
fn parse_exact_standard_g_valid() {
    let cases: [(&str, TimeSpan); 13] = [
        ("12", TimeSpan::from_ticks(10_368_000_000_000)),
        ("-12", -TimeSpan::from_ticks(10_368_000_000_000)),
        ("12:34", TimeSpan::from_ticks(452_400_000_000)),
        ("-12:34", -TimeSpan::from_ticks(452_400_000_000)),
        ("1:2:.3", TimeSpan::from_ticks(37_203_000_000)),
        ("-1:2:.3", -TimeSpan::from_ticks(37_203_000_000)),
        ("12:24:02", TimeSpan::from_ticks(446_420_000_000)),
        ("12:24:02.123", TimeSpan::from_ticks(446_421_230_000)),
        ("-12:24:02.123", -TimeSpan::from_ticks(446_421_230_000)),
        ("1:2:3:.4", TimeSpan::from_ticks(937_804_000_000)),
        ("-1:2:3:.4", -TimeSpan::from_ticks(937_804_000_000)),
        ("1:12:24:02", TimeSpan::from_ticks(1_310_420_000_000)),
        ("-01:07:45:16.999", -TimeSpan::from_ticks(1_143_169_990_000)),
    ];

    for (input, expected) in cases {
        assert_eq!(
            Ok(expected),
            TimeSpan::parse_exact(input, "g", TimeSpanStyles::None),
            "parsing {input:?} against format \"g\""
        );
    }
}

/// Cf. TimeSpanTests.cs#L1185-L1188 (`ParseExact_Valid_TestData`'s `"G"` rows).
#[test]
fn parse_exact_standard_g_long_valid() {
    let cases: [(&str, TimeSpan); 2] = [
        ("1:12:24:02.243", TimeSpan::from_ticks(1_310_422_430_000)),
        ("-01:07:45:16.999", -TimeSpan::from_ticks(1_143_169_990_000)),
    ];

    for (input, expected) in cases {
        assert_eq!(
            Ok(expected),
            TimeSpan::parse_exact(input, "G", TimeSpanStyles::None),
            "parsing {input:?} against format \"G\""
        );
    }
}

/// `TimeSpanStyles` is interpreted only for custom formats, not the five standard
/// single-letter ones — C#'s dispatch (TimeSpanParse.cs#L1231-1241) never passes `styles`
/// into `TryParseTimeSpanConstant`/`TryParseTimeSpan` for `'c'`/`'t'`/`'T'`/`'g'`/`'G'`, so
/// `AssumeNegative` has no effect on them (cf. `TimeSpanTests.cs`'s `ParseExact` test,
/// which skips the `AssumeNegative` assertions entirely for these five formats,
/// TimeSpanTests.cs#L1225-1229).
#[test]
fn parse_exact_standard_ignores_styles() {
    // "G" only accepts the full "d:h:m:s.f" shape, unlike the other four, so it needs its
    // own representative input rather than sharing "12:24:02" with the rest.
    let cases: [(&str, &str, TimeSpan); 5] = [
        ("12:24:02", "c", TimeSpan::from_ticks(446_420_000_000)),
        ("12:24:02", "t", TimeSpan::from_ticks(446_420_000_000)),
        ("12:24:02", "T", TimeSpan::from_ticks(446_420_000_000)),
        ("12:24:02", "g", TimeSpan::from_ticks(446_420_000_000)),
        (
            "1:12:24:02.243",
            "G",
            TimeSpan::from_ticks(1_310_422_430_000),
        ),
    ];

    for (input, format, expected) in cases {
        assert_eq!(
            Ok(expected),
            TimeSpan::parse_exact(input, format, TimeSpanStyles::AssumeNegative),
            "format {format:?} should ignore AssumeNegative"
        );
    }
}

/// Cf. TimeSpanTests.cs#L1252-L1274 (`ParseExact_Invalid_TestData`'s format-agnostic rows
/// plus the standard single-letter-format rows), restricted to rows usable without a
/// `null` `string` (no `&str` equivalent).
#[test]
fn parse_exact_standard_invalid() {
    let cases: [(&str, &str, TimeSpanError); 16] = [
        ("", "c", TimeSpanError::InvalidFormat),
        ("-", "c", TimeSpanError::InvalidFormat),
        ("garbage", "c", TimeSpanError::InvalidFormat),
        ("24:24:02", "c", TimeSpanError::Overflow),
        ("1:60:02", "c", TimeSpanError::Overflow),
        ("1:59:60", "c", TimeSpanError::Overflow),
        ("1.24:59:02", "c", TimeSpanError::Overflow),
        ("1.2:60:02", "c", TimeSpanError::Overflow),
        ("1?59:02", "c", TimeSpanError::InvalidFormat),
        ("1:59?02", "c", TimeSpanError::InvalidFormat),
        ("1:59:02?123", "c", TimeSpanError::InvalidFormat),
        ("1:12:24:02", "c", TimeSpanError::InvalidFormat),
        ("12:61:02", "g", TimeSpanError::Overflow),
        ("1.12:24:02", "g", TimeSpanError::InvalidFormat),
        ("1:07:45:16.99999999", "G", TimeSpanError::Overflow),
        ("1:12:24:02", "G", TimeSpanError::InvalidFormat),
    ];

    for (input, format, expected_err) in cases {
        assert_eq!(
            Err(expected_err),
            TimeSpan::parse_exact(input, format, TimeSpanStyles::None),
            "parsing {input:?} against format {format:?}"
        );
    }
}

/// Cf. `TimeSpanTests.cs`'s `ParseExactTest_Valid` body (TimeSpanTests.cs#L1209-1234),
/// which re-asserts every `ParseExact_Valid_TestData` row against the single-format-wrapped-
/// in-an-array overload (`TimeSpan.ParseExact(input, new string[] { format }, ...)`) too —
/// a single-element array must behave identically to the plain single-format overload.
#[test]
fn parse_exact_multiple_single_format_matches_parse_exact() {
    let cases: [(&str, &str, TimeSpan); 5] = [
        (
            "12.23:32:43",
            r"dd\.h\:m\:s",
            TimeSpan::from_ticks(11_215_630_000_000),
        ),
        ("3", "%h", TimeSpan::from_ticks(108_000_000_000)),
        ("678", "fff", TimeSpan::from_ticks(6_780_000)),
        ("1.12:24:02", "c", TimeSpan::from_ticks(1_310_420_000_000)),
        ("12:24:02", "g", TimeSpan::from_ticks(446_420_000_000)),
    ];

    for (input, format, expected) in cases {
        assert_eq!(
            Ok(expected),
            TimeSpan::parse_exact_multiple(input, &[format], TimeSpanStyles::None),
            "parsing {input:?} against single-element formats array {format:?}"
        );
    }
}

/// Cf. `TimeSpanTests.cs`'s `ParseExactTest_Valid` body (TimeSpanTests.cs#L1234, #L1239):
/// `TimeSpanStyles.AssumeNegative` is honored the same way through the array overload as
/// through the single-format overload (for the custom-format-string rows it applies to).
#[test]
fn parse_exact_multiple_assume_negative() {
    let cases: [(&str, &str, TimeSpan); 2] = [
        (
            "12.23:32:43",
            r"dd\.h\:m\:s",
            TimeSpan::from_ticks(11_215_630_000_000),
        ),
        ("3", "%h", TimeSpan::from_ticks(108_000_000_000)),
    ];

    for (input, format, expected) in cases {
        assert_eq!(
            Ok(-expected),
            TimeSpan::parse_exact_multiple(input, &[format], TimeSpanStyles::AssumeNegative),
            "parsing {input:?} against single-element formats array {format:?} with \
             AssumeNegative"
        );
    }
}

/// Cf. `TryParseExactMultipleTimeSpan` (TimeSpanParse.cs#L1662-1703): formats are tried in
/// array order, and the first one that matches wins. `"%h"` against `"3"` fails outright
/// (no `:` literal for `%h` to match against, so it's not just "wrong interpretation" —
/// `hh\:mm\:ss` requires two digits then a literal `:`, which a bare `"3"` doesn't have),
/// so the array must fall through to `"%h"` and succeed there.
#[test]
fn parse_exact_multiple_tries_formats_in_order() {
    assert_eq!(
        Ok(TimeSpan::from_ticks(108_000_000_000)),
        TimeSpan::parse_exact_multiple("3", &[r"hh\:mm\:ss", "%h"], TimeSpanStyles::None),
        "first format should fail to match, falling through to the second"
    );
}

/// Cf. `TryParseExactMultipleTimeSpan` (TimeSpanParse.cs#L1662-1703): order-sensitivity is
/// also observable in *which* value results, not just whether parsing succeeds — `"%h"` and
/// `"%m"` both accept a single digit, but interpret it differently, so swapping which comes
/// first in the array changes the parsed result.
#[test]
fn parse_exact_multiple_first_match_determines_interpretation() {
    assert_eq!(
        Ok(TimeSpan::from_ticks(108_000_000_000)),
        TimeSpan::parse_exact_multiple("3", &["%h", "%m"], TimeSpanStyles::None),
    );
    assert_eq!(
        Ok(TimeSpan::from_ticks(1_800_000_000)),
        TimeSpan::parse_exact_multiple("3", &["%m", "%h"], TimeSpanStyles::None),
    );
}

/// Cf. `TryParseExactMultipleTimeSpan` (TimeSpanParse.cs#L1662-1703): `formats.Length == 0`
/// is a distinct `SetNoFormatSpecifierFailure` bad-format failure — there's no `&str`
/// equivalent of C#'s separate `formats == null` -> `ArgumentNullException` case in this
/// crate (a `&[&str]` can't be null), but the empty-slice case still applies and, like every
/// other format failure in this crate, maps to `TimeSpanError::InvalidFormat`.
#[test]
fn parse_exact_multiple_empty_formats_array() {
    assert_eq!(
        Err(TimeSpanError::InvalidFormat),
        TimeSpan::parse_exact_multiple("12:34:56", &[], TimeSpanStyles::None),
    );
}

/// Cf. `TryParseExactMultipleTimeSpan` (TimeSpanParse.cs#L1662-1703): `input.Length == 0` is
/// an unconditional bad-format failure, checked before `formats` is even inspected — unlike
/// `TryParseExactTimeSpan` (the single-format overload, TimeSpanParse.cs#L1228-1247), which
/// has no such check at all. So `""` against a format that *would* successfully match empty
/// input (a custom format consisting solely of an empty quoted literal, e.g. `"''"`) must
/// still fail here, even though `TimeSpan::parse_exact("", "''", ..)` alone succeeds.
#[test]
fn parse_exact_multiple_empty_input_rejected_unconditionally() {
    assert_eq!(
        Ok(TimeSpan::ZERO),
        TimeSpan::parse_exact("", "''", TimeSpanStyles::None),
        "sanity check: the single-format overload accepts empty input against this format"
    );
    assert_eq!(
        Err(TimeSpanError::InvalidFormat),
        TimeSpan::parse_exact_multiple("", &["''"], TimeSpanStyles::None),
        "the array overload must reject empty input unconditionally, before ever trying a \
         format"
    );
}

/// Cf. `TryParseExactMultipleTimeSpan` (TimeSpanParse.cs#L1662-1703): an empty format string
/// anywhere in the array (`string.IsNullOrEmpty(format)`) is an immediate
/// `SetBadFormatSpecifierFailure`, returned right away rather than being skipped in favor of
/// a later entry that would otherwise have matched — this is the one case in the loop that
/// doesn't fall through to try the next format.
#[test]
fn parse_exact_multiple_empty_format_stops_immediately() {
    assert_eq!(
        Err(TimeSpanError::InvalidFormat),
        TimeSpan::parse_exact_multiple("3", &["", "%h"], TimeSpanStyles::None),
        "an empty format entry must fail immediately, not be skipped in favor of a later \
         match"
    );
}

/// Cf. `TimeSpanTests.cs`'s `ParseExactTest_Invalid` body (TimeSpanTests.cs#L1313-1315):
/// `exceptionTypeMultiple = exceptionType == typeof(OverflowException) ... ?
/// typeof(FormatException) : exceptionType` — `TryParseExactMultipleTimeSpan`'s per-format
/// attempts always run with `throwOnFailure: false` (a fresh, independent `TimeSpanResult`
/// each time), so an individual attempt's `OverflowException` is discarded exactly like an
/// individual attempt's `FormatException` would be; only the generic `SetBadTimeSpanFailure`
/// bad-format failure surfaces once every format in the array has failed. So even a
/// single-element array around a format that would overflow on its own turns that Overflow
/// into InvalidFormat here, unlike `TimeSpan::parse_exact` on the same input/format pair.
#[test]
fn parse_exact_multiple_overflow_becomes_invalid_format() {
    assert_eq!(
        Err(TimeSpanError::Overflow),
        TimeSpan::parse_exact("12.35:32:43", r"dd\.h\:m\:s", TimeSpanStyles::None),
        "sanity check: the single-format overload reports Overflow directly"
    );
    assert_eq!(
        Err(TimeSpanError::InvalidFormat),
        TimeSpan::parse_exact_multiple("12.35:32:43", &[r"dd\.h\:m\:s"], TimeSpanStyles::None),
        "the array overload must not leak the inner Overflow — it's swallowed into the \
         generic bad-format failure once every format has failed"
    );
}

/// Cf. `TryParseExactMultipleTimeSpan` (TimeSpanParse.cs#L1662-1703): when no format in the
/// array matches, the loop falls off the end into `SetBadTimeSpanFailure`.
#[test]
fn parse_exact_multiple_no_format_matches() {
    assert_eq!(
        Err(TimeSpanError::InvalidFormat),
        TimeSpan::parse_exact_multiple("garbage", &["%h", "%m", "%s"], TimeSpanStyles::None),
    );
}
