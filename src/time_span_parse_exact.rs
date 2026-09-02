//! `TimeSpan::parse_exact` (mirrors C#'s `ParseExact(string, string, IFormatProvider?,
//! TimeSpanStyles)`/`TryParseExact`, single format-string overload).
//!
//! `TimeSpan::parse_exact`'s top-level dispatch lives here: a 1-character `format` routes to
//! one of the single-letter standard formats (`"c"`/`"t"`/`"T"` via
//! `time_span_parse_constant::parse_constant`, `"g"`/`"G"` via
//! `time_span_parse::parse_general`); anything longer goes through this module's own
//! custom-format-string mini-language, ported from `TimeSpanParse.cs`'s `TryParseByFormat`
//! and its helpers (`ParseExactDigits`, `ParseExactLiteral`), plus the pieces of
//! `DateTimeFormat.cs`/`DateTimeParse.cs` it calls into (`ParseRepeatPattern`,
//! `ParseNextChar`, `TryParseQuoteString`) — these are shared with `DateTime`'s
//! custom-format parser upstream, so they don't have a `TimeSpan`-specific home in C#
//! either.
//!
//! The custom-format-string path is a genuinely different algorithm from
//! `time_span_parse.rs`'s standard-format path: it walks the *format* string
//! character-by-character (via `TimeSpanTokenizer`'s char-level `NextChar`/`BackOne`, not
//! its token-level `GetNextToken`), matching each format specifier against the input
//! directly, rather than tokenizing the input up front and pattern-matching the resulting
//! shape. The two paths do share `TryTimeToTicks` (`time_span_parse::time_to_ticks`) for the
//! final bounds-checked tick computation.
//!
//! `TimeSpan::parse_exact_multiple` (the multi-format-string-array overload set —
//! `ParseExactMultiple`/`TryParseExactMultiple`) also lives here: it's a thin driver over
//! this module's own `parse_exact`, ported from `TryParseExactMultipleTimeSpan`
//! (TimeSpanParse.cs#L1662-1703).
//!
//! Deferred (see `TimeSpan::parse_exact`'s doc comment for the full list): all
//! `IFormatProvider`/culture handling.

use crate::time_span::TimeSpanStyles;
use crate::time_span_parse::{NumTok, time_to_ticks};
use crate::{TimeSpan, TimeSpanError};

/// Cf. `DateTimeFormat.MaxSecondsFractionDigits` (DateTimeFormat.cs#L127), reused by
/// `TryParseByFormat`'s `'f'`/`'F'` cases (TimeSpanParse.cs#L1302-1319).
const MAX_FRACTION_DIGITS: u32 = 7;

/// Char-level scanner over the *input* string, walked directly by the format-string loop
/// below (one call per format specifier) rather than pre-tokenized.
///
/// Cf. `TimeSpanTokenizer`'s char-level surface — `NextChar`/`BackOne`/`EOL`
/// (TimeSpanParse.cs#L165-176, #L258-271); `GetNextToken` (the token-level surface used by
/// `time_span_parse.rs` instead) has no equivalent here.
struct CharTokenizer {
    chars: Vec<char>,
    /// Index of the last character returned by `next_char`. Starts at `-1` (mirroring
    /// `new TimeSpanTokenizer(input, -1)`, TimeSpanParse.cs#L1267) so the first `next_char`
    /// call lands on index `0`.
    pos: i64,
}

impl CharTokenizer {
    fn new(input: &str) -> Self {
        CharTokenizer {
            chars: input.chars().collect(),
            pos: -1,
        }
    }

    /// Cf. TimeSpanTokenizer.NextChar (TimeSpanParse.cs#L265-271): returns `'\0'` once past
    /// the end, rather than an `Option`, matching upstream's sentinel-character approach (no
    /// call site here treats `'\0'` as a legitimate format/literal character, so the
    /// sentinel can never be mistaken for real input).
    fn next_char(&mut self) -> char {
        self.pos += 1;
        let idx = self.pos;
        // `idx >= 0` (checked below) rules out sign loss. Truncation: `idx` grows by 1
        // per call, driven by walking `format`/matching digits against `input` — both
        // real `&str`s a caller actually allocated, so `idx` tracks their length. A
        // format/input string long enough to push `idx` anywhere near `u32::MAX` would
        // need multiple gigabytes just to exist as a `&str`/`Vec<char>` in the first
        // place, and (per `TimeSpanTokenizer`'s own `int`-typed `_pos`, TimeSpanParse.cs
        // relies on C# strings being `int`-length-capped, ~2^31) has no upstream C#
        // counterpart input to even diverge from.
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "idx >= 0 rules out sign loss; idx tracks a real input/format string's \
                      length, never remotely close to u32::MAX in any realistic invocation"
        )]
        if idx >= 0 && (idx as usize) < self.chars.len() {
            self.chars[idx as usize]
        } else {
            '\0'
        }
    }

    /// Cf. TimeSpanTokenizer.BackOne (TimeSpanParse.cs#L260-263) — a no-op at `pos == 0`,
    /// matched here rather than "fixed"; that's upstream's actual behavior, not a bug this
    /// port introduces.
    fn back_one(&mut self) {
        if self.pos > 0 {
            self.pos -= 1;
        }
    }

    /// Cf. TimeSpanTokenizer.EOL (TimeSpanParse.cs#L258).
    // `chars: Vec<char>` can never exceed `isize::MAX` bytes (Rust's allocator invariant
    // enforced on every push/collect) and `char` is always 4 bytes, so `chars.len() <=
    // isize::MAX / 4` — safely below `i64::MAX` on any platform.
    #[allow(
        clippy::cast_possible_wrap,
        reason = "a Vec<char> can never hold enough elements (isize::MAX/4, char being 4 \
                  bytes) for this usize -> i64 widening to wrap"
    )]
    fn eol(&self) -> bool {
        self.pos >= self.chars.len() as i64 - 1
    }
}

/// Reads up to `max_digit_length` ASCII digits, returning (leading zero count, value,
/// whether at least `min_digit_length` digits were read). The zero/value/consumed-so-far
/// outputs are meaningful even when the third element is `false` — callers that ignore it
/// (the `'F'` case) rely on that, exactly as `ParseExactDigits`'s C# `out` parameters are
/// still assigned on a `false` return.
///
/// Cf. `ParseExactDigits`, 2-min/max-argument overload (TimeSpanParse.cs#L1424-1446)
fn parse_exact_digits(tokenizer: &mut CharTokenizer, min: u32, max: u32) -> (u32, i64, bool) {
    let mut result: i64 = 0;
    let mut zeroes: u32 = 0;
    let mut token_length: u32 = 0;

    while token_length < max {
        let ch = tokenizer.next_char();
        if !ch.is_ascii_digit() {
            tokenizer.back_one();
            break;
        }
        result = result * 10 + i64::from(ch.to_digit(10).unwrap());
        if result == 0 {
            zeroes += 1;
        }
        token_length += 1;
    }

    (zeroes, result, token_length >= min)
}

/// Cf. `ParseExactDigits`, 1-`min`-argument overload (TimeSpanParse.cs#L1418-1422) — used by
/// the `'h'`/`'m'`/`'s'` cases, where a single-character specifier (e.g. `"h"`) accepts 1 or
/// 2 digits but a repeated one (e.g. `"hh"`) requires exactly that many.
fn digits_1_or_2(tokenizer: &mut CharTokenizer, min: u32) -> (i64, bool) {
    let max = if min == 1 { 2 } else { min };
    let (_, value, ok) = parse_exact_digits(tokenizer, min, max);
    (value, ok)
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

/// Extracts a quoted literal (`'...'`/`"..."`) starting at `pos` in the *format* string,
/// unescaping `\`-escaped characters within the quotes. Returns `(literal text, format
/// characters consumed including both quote characters)`, or `None` if the closing quote is
/// missing or a `\` appears at the very end of the format string.
///
/// Cf. DateTimeParse.TryParseQuoteString (DateTimeParse.cs#L4585-4636) — shared by
/// `DateTime`'s custom-format parser too, hence living in `DateTimeParse.cs` upstream
/// rather than `TimeSpanParse.cs`.
fn try_parse_quote_string(format: &[char], pos: usize) -> Option<(String, usize)> {
    let begin_pos = pos;
    let quote_char = format[pos];
    let mut i = pos + 1;
    let mut result = String::new();
    let mut found_quote = false;

    while i < format.len() {
        let ch = format[i];
        i += 1;
        if ch == quote_char {
            found_quote = true;
            break;
        } else if ch == '\\' {
            if i < format.len() {
                result.push(format[i]);
                i += 1;
            } else {
                return None;
            }
        } else {
            result.push(ch);
        }
    }

    if !found_quote {
        return None;
    }

    Some((result, i - begin_pos))
}

/// Matches `literal` character-by-character against the input, via the tokenizer.
///
/// Cf. `ParseExactLiteral` (TimeSpanParse.cs#L1448-1460)
fn parse_exact_literal(tokenizer: &mut CharTokenizer, literal: &str) -> bool {
    for expected in literal.chars() {
        if expected != tokenizer.next_char() {
            return false;
        }
    }
    true
}

/// Parses `input` against a custom format string. See `TimeSpan::parse_exact`'s doc comment
/// for the full scope (what's covered, what's deferred).
///
/// Cf. `TryParseByFormat` (TimeSpanParse.cs#L1250-1416)
#[allow(
    clippy::too_many_lines,
    reason = "a character-by-character tokenizer match mirroring C#'s equally-long \
              TryParseByFormat switch statement (TimeSpanParse.cs#L1250-1416) \
              one-to-one; splitting it up would obscure that direct correspondence \
              rather than clarify anything"
)]
pub(crate) fn parse_exact(
    input: &str,
    format: &str,
    styles: TimeSpanStyles,
) -> Result<TimeSpan, TimeSpanError> {
    let format_chars: Vec<char> = format.chars().collect();

    // Cf. TryParseExactTimeSpan (TimeSpanParse.cs#L1228-1247): `format.Length == 0` is
    // always a bad-format failure; `format.Length == 1` dispatches to the single-letter
    // standard formats ('c'/'t'/'T'/'g'/'G', each parsed by a *different* algorithm than
    // the custom-format tokenizer below — see `time_span_parse_constant.rs` for 'c'/'t'/'T'
    // and `time_span_parse::parse_general` for 'g'/'G'). `styles` is never consulted for any
    // of the five: TimeSpanParse.cs#L1237-1241's `switch` arms don't pass it through to
    // `TryParseTimeSpanConstant`/`TryParseTimeSpan` at all.
    if format_chars.is_empty() {
        return Err(TimeSpanError::InvalidFormat);
    }
    if format_chars.len() == 1 {
        return match format_chars[0] {
            'c' | 't' | 'T' => crate::time_span_parse_constant::parse_constant(input),
            'g' => crate::time_span_parse::parse_general(input, false),
            'G' => crate::time_span_parse::parse_general(input, true),
            _ => Err(TimeSpanError::InvalidFormat),
        };
    }

    let mut tokenizer = CharTokenizer::new(input);

    let mut seen_dd = false;
    let mut seen_hh = false;
    let mut seen_mm = false;
    let mut seen_ss = false;
    let mut seen_ff = false;

    let mut dd: i64 = 0;
    let mut hh: i64 = 0;
    let mut mm: i64 = 0;
    let mut ss: i64 = 0;
    let mut leading_zeroes: u32 = 0;
    let mut ff: i64 = 0;

    let mut i = 0usize;
    while i < format_chars.len() {
        let ch = format_chars[i];

        // Each arm below evaluates its digit/literal match eagerly (rather than
        // short-circuiting on `seen_*`/length checks the way the C# `||` chain does) since
        // every failure path returns immediately: whatever the eager match consumed from
        // the tokenizer is never observed afterward, so the two orders are behaviorally
        // equivalent for every terminating path.
        let token_len: usize = match ch {
            'h' => {
                let len = parse_repeat_pattern(&format_chars, i, ch);
                // Per the comment above: even if `len` were huge enough to truncate this
                // cast, `len > 2` below (using the real, untruncated `usize`) still
                // rejects the parse — the eager match's result is discarded either way.
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "len > 2 below always rejects an oversized len using the real \
                              usize value, regardless of what this cast truncates to"
                )]
                let (value, ok) = digits_1_or_2(&mut tokenizer, len as u32);
                if len > 2 || seen_hh || !ok {
                    return Err(TimeSpanError::InvalidFormat);
                }
                hh = value;
                seen_hh = true;
                len
            }
            'm' => {
                let len = parse_repeat_pattern(&format_chars, i, ch);
                // Cf. the 'h' arm above: `len > 2` below always rejects on the real,
                // untruncated `usize`, so a truncated cast here can't change the outcome.
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "len > 2 below always rejects an oversized len using the real \
                              usize value, regardless of what this cast truncates to"
                )]
                let (value, ok) = digits_1_or_2(&mut tokenizer, len as u32);
                if len > 2 || seen_mm || !ok {
                    return Err(TimeSpanError::InvalidFormat);
                }
                mm = value;
                seen_mm = true;
                len
            }
            's' => {
                let len = parse_repeat_pattern(&format_chars, i, ch);
                // Cf. the 'h' arm above: `len > 2` below always rejects on the real,
                // untruncated `usize`, so a truncated cast here can't change the outcome.
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "len > 2 below always rejects an oversized len using the real \
                              usize value, regardless of what this cast truncates to"
                )]
                let (value, ok) = digits_1_or_2(&mut tokenizer, len as u32);
                if len > 2 || seen_ss || !ok {
                    return Err(TimeSpanError::InvalidFormat);
                }
                ss = value;
                seen_ss = true;
                len
            }
            'f' => {
                let len = parse_repeat_pattern(&format_chars, i, ch);
                // Unlike the 'h'/'m'/'s' arms above, this cast *is* the only gate on
                // `len` here — but `len` is bounded by `format_chars.len() - i`, i.e. by
                // how many repeated 'f' characters actually exist in `format`. Reaching
                // anywhere near `u32::MAX` would require a `format` string of several
                // gigabytes, which — besides being unconstructable in any real
                // invocation — has no upstream counterpart to diverge from either:
                // `TimeSpanParse.cs`/`ParseRepeatPattern` operate on a C# `string`, whose
                // `Length` is a hard `int`-capped ~2^31, so no format string long enough
                // to trigger this even exists on the C# side.
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "len is bounded by format's own (realistically tiny) length; \
                              overflowing u32 needs a multi-gigabyte format string, which \
                              has no possible C# counterpart (string.Length is int-capped)"
                )]
                if len as u32 > MAX_FRACTION_DIGITS || seen_ff {
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
                let (zeroes, value, ok) =
                    parse_exact_digits(&mut tokenizer, len as u32, len as u32);
                if !ok {
                    return Err(TimeSpanError::InvalidFormat);
                }
                leading_zeroes = zeroes;
                ff = value;
                seen_ff = true;
                len
            }
            'F' => {
                let len = parse_repeat_pattern(&format_chars, i, ch);
                // Cf. the 'f' arm above: same len/format-length reasoning applies.
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "len is bounded by format's own (realistically tiny) length; \
                              overflowing u32 needs a multi-gigabyte format string, which \
                              has no possible C# counterpart (string.Length is int-capped)"
                )]
                if len as u32 > MAX_FRACTION_DIGITS || seen_ff {
                    return Err(TimeSpanError::InvalidFormat);
                }
                // Cf. TimeSpanParse.cs#L1317: the success/failure return of ParseExactDigits
                // is intentionally discarded here — 'F' digits are optional (0..=len of
                // them may actually be present), unlike 'f'.
                //
                // Cf. the 'f' arm above: same len/format-length reasoning applies to both
                // casts here.
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "len is bounded by format's own (realistically tiny) length; \
                              overflowing u32 needs a multi-gigabyte format string, which \
                              has no possible C# counterpart (string.Length is int-capped)"
                )]
                let (zeroes, value, _ok) =
                    parse_exact_digits(&mut tokenizer, len as u32, len as u32);
                leading_zeroes = zeroes;
                ff = value;
                seen_ff = true;
                len
            }
            'd' => {
                let len = parse_repeat_pattern(&format_chars, i, ch);
                // Cf. the comment above the 'h'/'m'/'s' arms: even if `len` were huge
                // enough to truncate this cast, `len > 8` below (using the real,
                // untruncated `usize`) still rejects the parse — the eager match's
                // result is discarded either way.
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "len > 8 below always rejects an oversized len using the real \
                              usize value, regardless of what this cast truncates to"
                )]
                let (min, max) = if len < 2 {
                    (1u32, 8u32)
                } else {
                    (len as u32, len as u32)
                };
                let (_, value, ok) = parse_exact_digits(&mut tokenizer, min, max);
                if len > 8 || seen_dd || !ok {
                    return Err(TimeSpanError::InvalidFormat);
                }
                dd = value;
                seen_dd = true;
                len
            }
            '\'' | '"' => {
                let (literal, consumed) =
                    try_parse_quote_string(&format_chars, i).ok_or(TimeSpanError::InvalidFormat)?;
                if !parse_exact_literal(&mut tokenizer, &literal) {
                    return Err(TimeSpanError::InvalidFormat);
                }
                consumed
            }
            '%' => {
                // Cf. TimeSpanParse.cs#L1346-1364: '%' at the end of the format string, or
                // "%%", are both bad-format failures; otherwise it's inert — just consumed
                // and skipped, letting the next format character be processed normally.
                match format_chars.get(i + 1).copied() {
                    Some(next) if next != '%' => 1,
                    _ => return Err(TimeSpanError::InvalidFormat),
                }
            }
            '\\' => {
                // Cf. TimeSpanParse.cs#L1366-1380: '\' escapes the following format
                // character into a literal that must match the next input character
                // exactly.
                match format_chars.get(i + 1).copied() {
                    Some(next) if tokenizer.next_char() == next => 2,
                    _ => return Err(TimeSpanError::InvalidFormat),
                }
            }
            // Cf. TimeSpanParse.cs#L1382-1383: any other format character is unquoted,
            // unescaped literal text, which custom formats don't allow — every literal
            // character must be quoted or `\`-escaped.
            _ => return Err(TimeSpanError::InvalidFormat),
        };

        i += token_len;
    }

    // Cf. TimeSpanParse.cs#L1390-1394: the custom format must consume the entire input.
    if !tokenizer.eol() {
        return Err(TimeSpanError::InvalidFormat);
    }

    // Cf. TimeSpanParse.cs#L1396: `TimeSpanStyles` is a bitflag enum in C# with only two
    // meaningful states here; represented in Rust as a plain 2-variant enum (see
    // `TimeSpanStyles`'s doc comment), so equality is the direct equivalent of the C#
    // `(styles & TimeSpanStyles.AssumeNegative) == 0` bit test.
    let positive = styles == TimeSpanStyles::None;

    let days = NumTok {
        value: dd,
        zeroes: 0,
    };
    let hours = NumTok {
        value: hh,
        zeroes: 0,
    };
    let minutes = NumTok {
        value: mm,
        zeroes: 0,
    };
    let seconds = NumTok {
        value: ss,
        zeroes: 0,
    };
    let fraction = NumTok {
        value: ff,
        zeroes: leading_zeroes,
    };

    let ticks = time_to_ticks(positive, days, hours, minutes, seconds, fraction)?;

    // Cf. TimeSpanParse.cs#L1404-1409: unlike the standard-format path's shared tail (e.g.
    // ProcessTerminal_D, TimeSpanParse.cs#L1211-1218, ported as `time_span_parse::finish`),
    // which re-checks for overflow after negating, `TryParseByFormat` negates with a plain
    // `ticks = -ticks` and no further check. `wrapping_neg` mirrors that: for the largest
    // magnitudes this can legitimately produce a bogus result rather than an overflow error
    // when `styles` is `AssumeNegative` — a real upstream quirk (the "positive && result <
    // 0" overflow guard inside `TryTimeToTicks` only fires when `positive` is `true`, i.e.
    // never for `AssumeNegative` input), not something this port introduces or "fixes".
    let ticks = if positive {
        ticks
    } else {
        ticks.wrapping_neg()
    };

    Ok(TimeSpan::from_ticks(ticks))
}

/// Tries each format in `formats` against `input` in order, returning the first successful
/// parse. See `TimeSpan::parse_exact_multiple`'s doc comment for the full scope.
///
/// Cf. `TryParseExactMultipleTimeSpan` (TimeSpanParse.cs#L1662-1703)
pub(crate) fn parse_exact_multiple(
    input: &str,
    formats: &[&str],
    styles: TimeSpanStyles,
) -> Result<TimeSpan, TimeSpanError> {
    // Cf. TimeSpanParse.cs#L1664-1673: `formats == null` has no `&str` equivalent (a
    // `&[&str]` can't be null), so only the `input.Length == 0` and `formats.Length == 0`
    // halves of upstream's three upfront checks apply here. The `input.Length == 0` check is
    // unconditional — it runs before `formats` is even inspected, and has no counterpart in
    // `parse_exact`/`TryParseExactTimeSpan` (the single-format overload), which happily
    // parses `""` against a format that matches empty input (e.g. a custom format consisting
    // solely of an empty quoted literal, `"''"`). The array overload's blanket rejection of
    // empty input is a real behavioral difference from the single-format overload, not a bug
    // to reconcile away.
    if input.is_empty() {
        return Err(TimeSpanError::InvalidFormat);
    }

    // Cf. TimeSpanParse.cs#L1674-1676, `SetNoFormatSpecifierFailure` — a bad-format failure
    // distinct from any individual format being bad.
    if formats.is_empty() {
        return Err(TimeSpanError::InvalidFormat);
    }

    // Cf. TimeSpanParse.cs#L1680-1696: each attempt uses a fresh, independent, non-throwing
    // result (`throwOnFailure: false`), so a failure on format `N` — including an
    // `OverflowException` from a valid-but-out-of-range match — never leaks into the attempt
    // on format `N+1`, and never surfaces past the loop either: only the generic
    // `SetBadTimeSpanFailure` bad-format failure below does, once every format has failed.
    for &format in formats {
        // Cf. TimeSpanParse.cs#L1684-1687: `string.IsNullOrEmpty(format)` is an immediate
        // bad-format-specifier failure — unlike every other failure in this loop, it returns
        // right away rather than falling through to try the next format.
        if format.is_empty() {
            return Err(TimeSpanError::InvalidFormat);
        }

        if let Ok(ts) = parse_exact(input, format, styles) {
            return Ok(ts);
        }
    }

    // Cf. TimeSpanParse.cs#L1698: every format failed.
    Err(TimeSpanError::InvalidFormat)
}
