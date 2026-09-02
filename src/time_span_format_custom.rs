//! Custom-format-string `TimeSpan::to_string_format` (mirrors C#'s `ToString(string?
//! format)`/`TryFormat`, custom-format-string half only — the standard `"c"`/`"t"`/`"T"`/
//! `"g"`/`"G"` single-character formats live in `time_span.rs` itself).
//!
//! Ports `TimeSpanFormat.cs`'s `FormatCustomized` and the pieces of `DateTimeFormat.cs`
//! it calls into (`ParseRepeatPattern`, `FormatDigits`, `FormatFraction`, `ParseQuoteString`),
//! plus `TimeSpanParse.Pow10UpToMaxFractionDigits` — these live outside `TimeSpanFormat.cs`
//! upstream because `DateTimeFormat`'s custom-format writer shares them with `DateTime`.
//!
//! This is the inverse of `time_span_parse_exact.rs`'s `parse_exact` (which ports
//! `TryParseByFormat`, the read side of the same mini-language): both walk the *format*
//! string character-by-character via a repeat-pattern/quote/escape tokenizer with the same
//! shape, but this module writes formatted digits into an output `String` instead of
//! matching digits out of an input string — a genuinely different direction, not
//! meaningfully shareable code between the two beyond the tokenizing *pattern*.
//!
//! Deferred (see `TimeSpan::to_string_format`'s doc comment): `IFormatProvider`/culture
//! handling (the decimal separator inside quoted/literal spans is irrelevant here since
//! custom format strings have no built-in fraction separator token — callers spell it out
//! themselves, e.g. `"dd\\.ss"` — but `TryFormatStandard`'s culture-aware decimal separator
//! for `"g"`/`"G"` remains out of scope, matching the rest of this crate).
//!
//! [`TimeSpan::try_format`] (the non-allocating `TChar`/UTF-8 `TryFormat` counterpart)
//! also routes custom format strings through [`format_customized`] rather than a
//! parallel buffer-writing tokenizer, then copies the resulting `String`'s bytes into
//! the caller's destination — see `try_format`'s doc comment for why that's a
//! deliberate tradeoff rather than an oversight.

use crate::TimeSpanError;
use crate::time_span::TimeSpan;

/// Cf. `DateTimeFormat.MaxSecondsFractionDigits` (DateTimeFormat.cs#L127), reused by
/// `FormatCustomized`'s `'f'`/`'F'` cases (TimeSpanFormat.cs#L353, #L367).
const MAX_FRACTION_DIGITS: u32 = 7;

/// `10.pow(pow)`, `pow` in `0..=MAX_FRACTION_DIGITS`.
///
/// Cf. `TimeSpanParse.Pow10UpToMaxFractionDigits` (TimeSpanParse.cs#L578-593) — a plain
/// lookup table upstream too, rather than computing the power on the fly.
fn pow10_up_to_max_fraction_digits(pow: u32) -> i64 {
    const POWERS_OF_TEN: [i64; 8] = [1, 10, 100, 1_000, 10_000, 100_000, 1_000_000, 10_000_000];
    POWERS_OF_TEN[pow as usize]
}

/// Counts how many characters starting at `pos` (inclusive) repeat `pattern_char`.
///
/// Cf. DateTimeFormat.ParseRepeatPattern (DateTimeFormat.cs#L197-205)
fn parse_repeat_pattern(format: &[char], pos: usize, pattern_char: char) -> usize {
    let mut index = pos + 1;
    while index < format.len() && format[index] == pattern_char {
        index += 1;
    }
    index - pos
}

/// Writes `value` (always non-negative, matching `FormatDigits`'s own precondition) zero-
/// padded to at least `minimum_length` digits. Rust's `{:0width$}` doesn't truncate a
/// value whose natural digit count exceeds `width`, matching `FormatDigits`/
/// `Number.UInt32ToDecChars`'s own "pad, never truncate" behavior.
///
/// Cf. DateTimeFormat.FormatDigits (DateTimeFormat.cs#L164-195)
fn format_digits(out: &mut String, value: i64, minimum_length: u32) {
    use std::fmt::Write;
    debug_assert!(value >= 0, "FormatDigits: value >= 0");
    let _ = write!(out, "{value:0width$}", width = minimum_length as usize);
}

/// Extracts a quoted literal (`'...'`/`"..."`) starting at `pos` in the format string,
/// unescaping `\`-escaped characters within the quotes. Returns `(literal text, format
/// characters consumed including both quote characters)`, or [`TimeSpanError::InvalidFormat`]
/// if the closing quote is missing or a `\` appears at the very end of the format string.
///
/// Cf. DateTimeFormat.ParseQuoteString (DateTimeFormat.cs#L284-337) — shared by `DateTime`'s
/// custom-format writer too, hence living in `DateTimeFormat.cs` upstream rather than
/// `TimeSpanFormat.cs`.
fn parse_quote_string(format: &[char], pos: usize) -> Result<(String, usize), TimeSpanError> {
    let format_len = format.len();
    let begin_pos = pos;
    let quote_char = format[pos];
    let mut i = pos + 1;
    let mut result = String::new();
    let mut found_quote = false;

    while i < format_len {
        let ch = format[i];
        i += 1;
        if ch == quote_char {
            found_quote = true;
            break;
        } else if ch == '\\' {
            if i < format_len {
                result.push(format[i]);
                i += 1;
            } else {
                return Err(TimeSpanError::InvalidFormat);
            }
        } else {
            result.push(ch);
        }
    }

    if !found_quote {
        return Err(TimeSpanError::InvalidFormat);
    }

    Ok((result, i - begin_pos))
}

/// Formats `ts` using a custom format string. See `TimeSpan::to_string_format`'s doc
/// comment for the full scope (what's covered, what's deferred).
///
/// Cf. TimeSpanFormat.cs#L296-455 (`FormatCustomized`)
#[allow(
    clippy::too_many_lines,
    reason = "a character-by-character tokenizer match mirroring C#'s equally-long \
              FormatCustomized switch statement (TimeSpanFormat.cs#L296-455) \
              one-to-one; splitting it up would obscure that direct correspondence \
              rather than clarify anything"
)]
pub(crate) fn format_customized(ts: TimeSpan, format: &str) -> Result<String, TimeSpanError> {
    let format_chars: Vec<char> = format.chars().collect();
    let ticks = ts.ticks();

    // Cf. TimeSpanFormat.cs#L301-312: `day`/`time` are computed as non-negative
    // magnitudes *before* the tokenizer loop runs, and no case in the switch below
    // ever writes a sign character. A negative `TimeSpan` formatted with a custom
    // format string therefore renders identically to its positive magnitude — a real
    // upstream quirk (unlike the standard `"c"`/`"g"`/`"G"` formats, which do prepend
    // `-`), not something this port introduces. `TicksPerDay` is well within `i64`
    // range for any representable `TimeSpan`, so unlike `Display`/`format_general`
    // (which negate the *whole* tick count and need `i128` to survive `TimeSpan::MIN`),
    // no widening is needed here: `day`/`time` are each derived from a division/modulo
    // performed *before* negation, so neither can land on `i64::MIN`.
    let mut day = ticks / TimeSpan::TICKS_PER_DAY;
    let mut time = ticks % TimeSpan::TICKS_PER_DAY;
    if ticks < 0 {
        day = -day;
        time = -time;
    }
    let hours = time / TimeSpan::TICKS_PER_HOUR % 24;
    let minutes = time / TimeSpan::TICKS_PER_MINUTE % 60;
    let seconds = time / TimeSpan::TICKS_PER_SECOND % 60;
    let fraction = time % TimeSpan::TICKS_PER_SECOND;

    let mut result = String::new();
    let mut i = 0usize;

    while i < format_chars.len() {
        let ch = format_chars[i];

        let token_len: usize = match ch {
            'h' => {
                let len = parse_repeat_pattern(&format_chars, i, ch);
                if len > 2 {
                    return Err(TimeSpanError::InvalidFormat);
                }
                // `len <= 2` is already established by the check just above.
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "len <= 2 is already checked above, so this cast is always exact"
                )]
                format_digits(&mut result, hours, len as u32);
                len
            }
            'm' => {
                let len = parse_repeat_pattern(&format_chars, i, ch);
                if len > 2 {
                    return Err(TimeSpanError::InvalidFormat);
                }
                // `len <= 2` is already established by the check just above.
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "len <= 2 is already checked above, so this cast is always exact"
                )]
                format_digits(&mut result, minutes, len as u32);
                len
            }
            's' => {
                let len = parse_repeat_pattern(&format_chars, i, ch);
                if len > 2 {
                    return Err(TimeSpanError::InvalidFormat);
                }
                // `len <= 2` is already established by the check just above.
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "len <= 2 is already checked above, so this cast is always exact"
                )]
                format_digits(&mut result, seconds, len as u32);
                len
            }
            'f' => {
                // The fraction of a second in single-digit precision. The remaining
                // digits are truncated.
                let len = parse_repeat_pattern(&format_chars, i, ch);
                // Unlike the 'h'/'m'/'s' arms above, this cast *is* the only gate on
                // `len` here — but `len` is bounded by `format_chars.len() - i`, i.e. by
                // how many repeated 'f' characters actually exist in `format`. Reaching
                // anywhere near `u32::MAX` would require a `format` string of several
                // gigabytes, which — besides being unconstructable in any real
                // invocation — has no upstream counterpart to diverge from either:
                // `TimeSpanFormat.cs`/`ParseRepeatPattern` operate on a C# `string`,
                // whose `Length` is a hard `int`-capped ~2^31, so no format string long
                // enough to trigger this even exists on the C# side.
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "len is bounded by format's own (realistically tiny) length; \
                              overflowing u32 needs a multi-gigabyte format string, which \
                              has no possible C# counterpart (string.Length is int-capped)"
                )]
                if len as u32 > MAX_FRACTION_DIGITS {
                    return Err(TimeSpanError::InvalidFormat);
                }
                // Cf. the allow above: same len/format-length reasoning applies to both
                // casts here.
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "len is bounded by format's own (realistically tiny) length; \
                              overflowing u32 needs a multi-gigabyte format string, which \
                              has no possible C# counterpart (string.Length is int-capped)"
                )]
                let tmp =
                    fraction / pow10_up_to_max_fraction_digits(MAX_FRACTION_DIGITS - len as u32);
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "len is bounded by format's own (realistically tiny) length; \
                              overflowing u32 needs a multi-gigabyte format string, which \
                              has no possible C# counterpart (string.Length is int-capped)"
                )]
                format_digits(&mut result, tmp, len as u32);
                len
            }
            'F' => {
                // Displays the most significant digits of the seconds fraction.
                // Nothing is displayed if the trimmed value is empty.
                let len = parse_repeat_pattern(&format_chars, i, ch);
                // Cf. the 'f' arm above: same len/format-length reasoning applies to
                // every cast of `len` in this arm.
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "len is bounded by format's own (realistically tiny) length; \
                              overflowing u32 needs a multi-gigabyte format string, which \
                              has no possible C# counterpart (string.Length is int-capped)"
                )]
                if len as u32 > MAX_FRACTION_DIGITS {
                    return Err(TimeSpanError::InvalidFormat);
                }
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "len is bounded by format's own (realistically tiny) length; \
                              overflowing u32 needs a multi-gigabyte format string, which \
                              has no possible C# counterpart (string.Length is int-capped)"
                )]
                let mut tmp =
                    fraction / pow10_up_to_max_fraction_digits(MAX_FRACTION_DIGITS - len as u32);
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "len is bounded by format's own (realistically tiny) length; \
                              overflowing u32 needs a multi-gigabyte format string, which \
                              has no possible C# counterpart (string.Length is int-capped)"
                )]
                let mut effective_digits = len as u32;
                while effective_digits > 0 {
                    if tmp % 10 == 0 {
                        tmp /= 10;
                        effective_digits -= 1;
                    } else {
                        break;
                    }
                }
                if effective_digits > 0 {
                    format_digits(&mut result, tmp, effective_digits);
                }
                len
            }
            'd' => {
                // tokenLen == 1 : Day as digits with no leading zero.
                // tokenLen == 2+: Day as digits with leading zero for single-digit days.
                let len = parse_repeat_pattern(&format_chars, i, ch);
                if len > 8 {
                    return Err(TimeSpanError::InvalidFormat);
                }
                // `len <= 8` is already established by the check just above.
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "len <= 8 is already checked above, so this cast is always exact"
                )]
                format_digits(&mut result, day, len as u32);
                len
            }
            '\'' | '"' => {
                let (literal, consumed) = parse_quote_string(&format_chars, i)?;
                result.push_str(&literal);
                consumed
            }
            '%' => {
                // Optional format character. For example, format string "%d" will
                // print day. Most of the cases, "%" can be ignored. "%" at the end of
                // the format string, or "%%", are both bad-format failures.
                match format_chars.get(i + 1).copied() {
                    Some(next) if next != '%' => {
                        result.push_str(&format_customized(ts, &next.to_string())?);
                        2
                    }
                    _ => return Err(TimeSpanError::InvalidFormat),
                }
            }
            '\\' => {
                // Escaped character. Can be used to insert a character into the
                // format string, e.g. "\d" inserts the literal character 'd'.
                match format_chars.get(i + 1).copied() {
                    Some(next) => {
                        result.push(next);
                        2
                    }
                    None => return Err(TimeSpanError::InvalidFormat),
                }
            }
            // Cf. TimeSpanFormat.cs#L449-451: any other format character is unquoted,
            // unescaped literal text, which custom formats don't allow — every literal
            // character must be quoted or `\`-escaped.
            _ => return Err(TimeSpanError::InvalidFormat),
        };

        i += token_len;
    }

    Ok(result)
}
