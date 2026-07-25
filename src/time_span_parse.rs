//! Invariant-culture `TimeSpan::from_str` (mirrors C#'s `Parse(string)`/`TryParse`).
//!
//! Ports the standard-format half of `TimeSpanParse.cs`
//! (`src/libraries/System.Private.CoreLib/src/System/Globalization/TimeSpanParse.cs`):
//! tokenizing into numbers/separators (`TimeSpanTokenizer`), then matching the
//! resulting shape against the "d.h:m:s.f" grammar (`ProcessTerminalState` and its
//! `ProcessTerminal_*` helpers). The custom-format-string half (`TryParseByFormat`) lives
//! in `time_span_parse_exact.rs` instead — a different, char-level tokenizer entirely; see
//! that module's doc comment. `IFormatProvider`/culture-aware parsing and the single-letter
//! standard formats (`"c"`/`"t"`/`"T"`/`"g"`/`"G"`) accepted by `ParseExact` remain out of
//! scope everywhere in this crate; see the `FromStr` impl's doc comment in `time_span.rs`
//! and `TimeSpan::parse_exact`'s doc comment.

use crate::{TimeSpan, TimeSpanError};

// Cf. TimeSpanParse.cs#L58-63
const MAX_DAYS: i64 = 10_675_199;
const MAX_HOURS: i64 = 23;
const MAX_MINUTES: i64 = 59;
const MAX_SECONDS: i64 = 59;
const MAX_FRACTION: i64 = 9_999_999;
const MAX_FRACTION_DIGITS: u32 = 7;

/// Matches the tokenizer's `(num & 0xF0000000) != 0` overflow guard (max representable
/// token value is 0x0FFFFFFF = 268,435,455).
///
/// Cf. TimeSpanParse.cs#L234
const TOKEN_MAX: i64 = 0x0FFF_FFFF;

const POWERS_OF_TEN: [i64; MAX_FRACTION_DIGITS as usize + 1] =
    [1, 10, 100, 1_000, 10_000, 100_000, 1_000_000, 10_000_000];

const POS_START: &str = "";
const NEG_START: &str = "-";
const HOUR_MINUTE_SEP: &str = ":";
const MINUTE_SECOND_SEP: &str = ":";
const SECOND_FRACTION_SEP: &str = ".";
const END: &str = "";
/// `MinuteSecondSep + SecondFractionSep`, used by the "h:m:.f" / "d.h:m:.f" shapes that
/// elide an explicit seconds component.
///
/// Cf. TimeSpanFormat.cs#L486
const APP_COMPAT_LITERAL: &str = ":.";

/// Whether `sep` is an acceptable days/hours separator.
///
/// C#'s invariant "c" format literals use '.' here (`TimeSpanFormat.FormatLiterals.InitInvariant`,
/// TimeSpanFormat.cs#L476-493), but `TimeSpanParse`'s standard-format `Parse`/`TryParse` also
/// unconditionally tries `DateTimeFormatInfo.FullTimeSpanPositivePattern`
/// (DateTimeFormatInfo.cs#L1706-1707), which hardcodes `"d':'h':'mm':'ss'..."` — ':' as the
/// days/hours separator — for *every* culture, including `CultureInfo.InvariantCulture`. Real
/// `Parse`/`TryParse` therefore accepts either separator (confirmed empirically: `"24:00:00"`
/// parses as 24 days rather than throwing on an out-of-range "24 hours", since the "h:m:s"
/// shape is tried first, fails its 0-23 hour bound, and falls back to "d:h:m").
fn is_day_hour_sep(sep: &str) -> bool {
    sep == "." || sep == ":"
}

/// A parsed numeric token: the value plus its count of leading zero digits (leading zeroes
/// matter for fractional-second normalization, where digit *count* — not just value —
/// determines the tick scale).
///
/// Cf. TimeSpanParse.cs's `TimeSpanToken` (TimeSpanParse.cs#L86-105)
///
/// `pub(crate)` (rather than private to this module): `time_span_parse_exact.rs`'s
/// custom-format-string tokenizer reuses this type plus [`time_to_ticks`] directly, so the
/// tick-bounds/fraction-normalization arithmetic (`TryTimeToTicks`) has exactly one Rust
/// implementation shared by both the standard-format and custom-format parsing paths, the
/// same way upstream shares a single `TryTimeToTicks` between them.
#[derive(Clone, Copy)]
pub(crate) struct NumTok {
    pub(crate) value: i64,
    pub(crate) zeroes: u32,
}

impl NumTok {
    pub(crate) const ZERO: NumTok = NumTok {
        value: 0,
        zeroes: 0,
    };
}

enum Tok {
    End,
    Num { value: i64, zeroes: u32 },
    Sep(String),
    Overflow,
}

/// Splits an already-trimmed input into a stream of number/separator tokens.
///
/// Cf. TimeSpanParse.cs's `TimeSpanTokenizer` (TimeSpanParse.cs#L165-272)
struct Lexer {
    chars: Vec<char>,
    pos: usize,
}

impl Lexer {
    fn new(s: &str) -> Self {
        Lexer {
            chars: s.chars().collect(),
            pos: 0,
        }
    }

    fn next(&mut self) -> Tok {
        let Some(&c) = self.chars.get(self.pos) else {
            return Tok::End;
        };

        if !c.is_ascii_digit() {
            let start = self.pos;
            self.pos += 1;
            while let Some(d) = self.chars.get(self.pos) {
                if d.is_ascii_digit() {
                    break;
                }
                self.pos += 1;
            }
            return Tok::Sep(self.chars[start..self.pos].iter().collect());
        }

        // Count leading zeroes (if any), then read the first significant digit.
        let mut zeroes = 0u32;
        self.pos += 1;
        if c == '0' {
            zeroes = 1;
            loop {
                match self.chars.get(self.pos) {
                    Some('0') => {
                        zeroes += 1;
                        self.pos += 1;
                    }
                    Some(d) if d.is_ascii_digit() => break,
                    _ => return Tok::Num { value: 0, zeroes },
                }
            }
        }

        let mut value = if zeroes > 0 {
            let d = self.chars[self.pos];
            self.pos += 1;
            d.to_digit(10).unwrap() as i64
        } else {
            c.to_digit(10).unwrap() as i64
        };

        while let Some(d) = self.chars.get(self.pos) {
            if !d.is_ascii_digit() {
                break;
            }
            value = value * 10 + d.to_digit(10).unwrap() as i64;
            self.pos += 1;
            if value > TOKEN_MAX {
                return Tok::Overflow;
            }
        }

        Tok::Num { value, zeroes }
    }
}

/// Parses `s` per invariant-culture `TimeSpan.Parse`/`TryParse` semantics.
///
/// Cf. TimeSpanParse.cs's `TryParseTimeSpan` (TimeSpanParse.cs#L694-727)
pub(crate) fn parse(s: &str) -> Result<TimeSpan, TimeSpanError> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err(TimeSpanError::InvalidFormat);
    }

    let mut lexer = Lexer::new(trimmed);
    let mut numbers: Vec<NumTok> = Vec::with_capacity(5);
    let mut seps: Vec<String> = Vec::with_capacity(6);
    let mut token_count = 0usize;
    let mut last_was_num = false;

    loop {
        match lexer.next() {
            Tok::End => break,
            Tok::Overflow => return Err(TimeSpanError::Overflow),
            Tok::Num { value, zeroes } => {
                if token_count == 0 {
                    add_sep(&mut seps, &mut token_count, String::new())?;
                }
                add_num(&mut numbers, &mut token_count, NumTok { value, zeroes })?;
                last_was_num = true;
            }
            Tok::Sep(text) => {
                add_sep(&mut seps, &mut token_count, text)?;
                last_was_num = false;
            }
        }
    }

    // The grammar always requires a trailing separator (possibly empty); synthesize one if
    // the input ended mid-number, e.g. "1:2:3" has no literal separator after the final "3".
    if last_was_num {
        add_sep(&mut seps, &mut token_count, String::new())?;
    }

    match numbers.len() {
        1 => process_d(&numbers, &seps),
        2 => process_hm(&numbers, &seps),
        3 => process_hm_s_d(&numbers, &seps),
        4 => process_hms_f_d(&numbers, &seps),
        5 => process_dhmsf(&numbers, &seps),
        _ => Err(TimeSpanError::InvalidFormat),
    }
}

/// Cf. TimeSpanRawInfo's `AddSep` (TimeSpanParse.cs#L451-470); the token/literal-count caps
/// mirror `MaxTokens`/`MaxLiteralTokens` (TimeSpanParse.cs#L398-399).
fn add_sep(
    seps: &mut Vec<String>,
    token_count: &mut usize,
    sep: String,
) -> Result<(), TimeSpanError> {
    if seps.len() >= 6 || *token_count >= 11 {
        return Err(TimeSpanError::InvalidFormat);
    }
    seps.push(sep);
    *token_count += 1;
    Ok(())
}

/// Cf. TimeSpanRawInfo's `AddNum` (TimeSpanParse.cs#L471-489); the token/numeric-count caps
/// mirror `MaxTokens`/`MaxNumericTokens` (TimeSpanParse.cs#L398, #L400).
fn add_num(
    numbers: &mut Vec<NumTok>,
    token_count: &mut usize,
    tok: NumTok,
) -> Result<(), TimeSpanError> {
    if numbers.len() >= 5 || *token_count >= 11 {
        return Err(TimeSpanError::InvalidFormat);
    }
    numbers.push(tok);
    *token_count += 1;
    Ok(())
}

/// Cf. ProcessTerminal_D (TimeSpanParse.cs#L1159-1225)
fn process_d(n: &[NumTok], s: &[String]) -> Result<TimeSpan, TimeSpanError> {
    let (s0, s1) = (s[0].as_str(), s[1].as_str());
    let positive = match (s0, s1) {
        (POS_START, END) => true,
        (NEG_START, END) => false,
        _ => return Err(TimeSpanError::InvalidFormat),
    };
    let ticks = time_to_ticks(
        positive,
        n[0],
        NumTok::ZERO,
        NumTok::ZERO,
        NumTok::ZERO,
        NumTok::ZERO,
    )?;
    finish(positive, ticks)
}

/// Cf. ProcessTerminal_HM (TimeSpanParse.cs#L1090-1156)
fn process_hm(n: &[NumTok], s: &[String]) -> Result<TimeSpan, TimeSpanError> {
    let (s0, s1, s2) = (s[0].as_str(), s[1].as_str(), s[2].as_str());
    let positive = match (s0, s1, s2) {
        (POS_START, HOUR_MINUTE_SEP, END) => true,
        (NEG_START, HOUR_MINUTE_SEP, END) => false,
        _ => return Err(TimeSpanError::InvalidFormat),
    };
    let ticks = time_to_ticks(
        positive,
        NumTok::ZERO,
        n[0],
        n[1],
        NumTok::ZERO,
        NumTok::ZERO,
    )?;
    finish(positive, ticks)
}

/// Tries each `(literal-match, positive, days, hours, minutes, seconds, fraction)` candidate
/// in order, mirroring the C# `overflow = overflow || !match` accumulation: a literal match
/// whose numeric bounds fail is remembered as an overflow rather than immediately reported,
/// so a later, still-untried candidate gets a chance to succeed.
///
/// Cf. ProcessTerminal_HM_S_D / ProcessTerminal_HMS_F_D / ProcessTerminal_DHMSF
fn process_ambiguous(
    candidates: &[(bool, bool, NumTok, NumTok, NumTok, NumTok, NumTok)],
) -> Result<TimeSpan, TimeSpanError> {
    let mut overflow = false;
    for &(literal_match, positive, days, hours, minutes, seconds, fraction) in candidates {
        if !literal_match {
            continue;
        }
        match time_to_ticks(positive, days, hours, minutes, seconds, fraction) {
            Ok(ticks) => return finish(positive, ticks),
            Err(TimeSpanError::Overflow) => overflow = true,
            Err(e) => return Err(e),
        }
    }
    Err(if overflow {
        TimeSpanError::Overflow
    } else {
        TimeSpanError::InvalidFormat
    })
}

/// Cf. ProcessTerminal_HM_S_D (TimeSpanParse.cs#L963-1087): "h:m:s", "d.h:m", or "h:m:.f".
fn process_hm_s_d(n: &[NumTok], s: &[String]) -> Result<TimeSpan, TimeSpanError> {
    let (s0, s1, s2, s3) = (s[0].as_str(), s[1].as_str(), s[2].as_str(), s[3].as_str());
    let zero = NumTok::ZERO;
    let candidates = [
        // HMS: h:m:s
        (
            s0 == POS_START && s1 == HOUR_MINUTE_SEP && s2 == MINUTE_SECOND_SEP && s3 == END,
            true,
            zero,
            n[0],
            n[1],
            n[2],
            zero,
        ),
        // DHM: d(.|:)h:m
        (
            s0 == POS_START && is_day_hour_sep(s1) && s2 == HOUR_MINUTE_SEP && s3 == END,
            true,
            n[0],
            n[1],
            n[2],
            zero,
            zero,
        ),
        // AppCompat: h:m:.f
        (
            s0 == POS_START && s1 == HOUR_MINUTE_SEP && s2 == APP_COMPAT_LITERAL && s3 == END,
            true,
            zero,
            n[0],
            n[1],
            zero,
            n[2],
        ),
        (
            s0 == NEG_START && s1 == HOUR_MINUTE_SEP && s2 == MINUTE_SECOND_SEP && s3 == END,
            false,
            zero,
            n[0],
            n[1],
            n[2],
            zero,
        ),
        (
            s0 == NEG_START && is_day_hour_sep(s1) && s2 == HOUR_MINUTE_SEP && s3 == END,
            false,
            n[0],
            n[1],
            n[2],
            zero,
            zero,
        ),
        (
            s0 == NEG_START && s1 == HOUR_MINUTE_SEP && s2 == APP_COMPAT_LITERAL && s3 == END,
            false,
            zero,
            n[0],
            n[1],
            zero,
            n[2],
        ),
    ];
    process_ambiguous(&candidates)
}

/// Cf. ProcessTerminal_HMS_F_D (TimeSpanParse.cs#L834-961): "h:m:s.f", "d.h:m:s", or "d.h:m:.f".
fn process_hms_f_d(n: &[NumTok], s: &[String]) -> Result<TimeSpan, TimeSpanError> {
    let (s0, s1, s2, s3, s4) = (
        s[0].as_str(),
        s[1].as_str(),
        s[2].as_str(),
        s[3].as_str(),
        s[4].as_str(),
    );
    let zero = NumTok::ZERO;
    let candidates = [
        // HMSF: h:m:s.f
        (
            s0 == POS_START
                && s1 == HOUR_MINUTE_SEP
                && s2 == MINUTE_SECOND_SEP
                && s3 == SECOND_FRACTION_SEP
                && s4 == END,
            true,
            zero,
            n[0],
            n[1],
            n[2],
            n[3],
        ),
        // DHMS: d(.|:)h:m:s
        (
            s0 == POS_START
                && is_day_hour_sep(s1)
                && s2 == HOUR_MINUTE_SEP
                && s3 == MINUTE_SECOND_SEP
                && s4 == END,
            true,
            n[0],
            n[1],
            n[2],
            n[3],
            zero,
        ),
        // AppCompat: d(.|:)h:m:.f
        (
            s0 == POS_START
                && is_day_hour_sep(s1)
                && s2 == HOUR_MINUTE_SEP
                && s3 == APP_COMPAT_LITERAL
                && s4 == END,
            true,
            n[0],
            n[1],
            n[2],
            zero,
            n[3],
        ),
        (
            s0 == NEG_START
                && s1 == HOUR_MINUTE_SEP
                && s2 == MINUTE_SECOND_SEP
                && s3 == SECOND_FRACTION_SEP
                && s4 == END,
            false,
            zero,
            n[0],
            n[1],
            n[2],
            n[3],
        ),
        (
            s0 == NEG_START
                && is_day_hour_sep(s1)
                && s2 == HOUR_MINUTE_SEP
                && s3 == MINUTE_SECOND_SEP
                && s4 == END,
            false,
            n[0],
            n[1],
            n[2],
            n[3],
            zero,
        ),
        (
            s0 == NEG_START
                && is_day_hour_sep(s1)
                && s2 == HOUR_MINUTE_SEP
                && s3 == APP_COMPAT_LITERAL
                && s4 == END,
            false,
            n[0],
            n[1],
            n[2],
            zero,
            n[3],
        ),
    ];
    process_ambiguous(&candidates)
}

/// Cf. ProcessTerminal_DHMSF (TimeSpanParse.cs#L767-831): "d.h:m:s.f".
fn process_dhmsf(n: &[NumTok], s: &[String]) -> Result<TimeSpan, TimeSpanError> {
    let (s0, s1, s2, s3, s4, s5) = (
        s[0].as_str(),
        s[1].as_str(),
        s[2].as_str(),
        s[3].as_str(),
        s[4].as_str(),
        s[5].as_str(),
    );
    let candidates = [
        (
            s0 == POS_START
                && is_day_hour_sep(s1)
                && s2 == HOUR_MINUTE_SEP
                && s3 == MINUTE_SECOND_SEP
                && s4 == SECOND_FRACTION_SEP
                && s5 == END,
            true,
            n[0],
            n[1],
            n[2],
            n[3],
            n[4],
        ),
        (
            s0 == NEG_START
                && is_day_hour_sep(s1)
                && s2 == HOUR_MINUTE_SEP
                && s3 == MINUTE_SECOND_SEP
                && s4 == SECOND_FRACTION_SEP
                && s5 == END,
            false,
            n[0],
            n[1],
            n[2],
            n[3],
            n[4],
        ),
    ];
    process_ambiguous(&candidates)
}

/// Cf. TryTimeToTicks (TimeSpanParse.cs#L595-625)
///
/// `pub(crate)`: shared with `time_span_parse_exact.rs` — see [`NumTok`]'s doc comment.
pub(crate) fn time_to_ticks(
    positive: bool,
    days: NumTok,
    hours: NumTok,
    minutes: NumTok,
    seconds: NumTok,
    fraction: NumTok,
) -> Result<i64, TimeSpanError> {
    if days.value > MAX_DAYS
        || hours.value > MAX_HOURS
        || minutes.value > MAX_MINUTES
        || seconds.value > MAX_SECONDS
    {
        return Err(TimeSpanError::Overflow);
    }
    let fraction_ticks = normalize_fraction(fraction)?;

    let ticks_ms =
        (days.value * 86_400 + hours.value * 3_600 + minutes.value * 60 + seconds.value) * 1_000;
    const MAX_MS: i64 = i64::MAX / TimeSpan::TICKS_PER_MILLISECOND;
    const MIN_MS: i64 = i64::MIN / TimeSpan::TICKS_PER_MILLISECOND;
    if !(MIN_MS..=MAX_MS).contains(&ticks_ms) {
        return Err(TimeSpanError::Overflow);
    }

    // Mirrors TimeSpanParse.cs's unchecked `long` arithmetic: for the largest valid-looking
    // inputs this multiply+add can legitimately wrap past i64::MAX, and the `positive &&
    // result < 0` check below is how upstream turns that wraparound into an overflow error
    // instead of silently accepting a bogus negative value.
    let result = ticks_ms
        .wrapping_mul(TimeSpan::TICKS_PER_MILLISECOND)
        .wrapping_add(fraction_ticks);
    if positive && result < 0 {
        return Err(TimeSpanError::Overflow);
    }
    Ok(result)
}

/// Cf. `TimeSpanToken.NormalizeAndValidateFraction` (TimeSpanParse.cs#L107-162)
fn normalize_fraction(tok: NumTok) -> Result<i64, TimeSpanError> {
    if tok.value == 0 {
        return Ok(0);
    }
    if tok.zeroes == 0 && tok.value > MAX_FRACTION {
        return Err(TimeSpanError::Overflow);
    }

    let total_digits = count_digits(tok.value) + tok.zeroes;

    match total_digits.cmp(&MAX_FRACTION_DIGITS) {
        std::cmp::Ordering::Equal => Ok(tok.value),
        std::cmp::Ordering::Less => {
            Ok(tok.value * POWERS_OF_TEN[(MAX_FRACTION_DIGITS - total_digits) as usize])
        }
        std::cmp::Ordering::Greater => {
            if tok.zeroes > MAX_FRACTION_DIGITS {
                return Ok(0);
            }
            // Unsigned division, rounding away from zero.
            let power = POWERS_OF_TEN[(total_digits - MAX_FRACTION_DIGITS) as usize] as u64;
            let value = (tok.value as u64 + power / 2) / power;
            Ok(value as i64)
        }
    }
}

fn count_digits(mut n: i64) -> u32 {
    debug_assert!(n > 0);
    let mut count = 0;
    while n > 0 {
        count += 1;
        n /= 10;
    }
    count
}

/// Applies the sign to a magnitude computed under a "positive" assumption.
///
/// Cf. the shared tail of each `ProcessTerminal_*` (e.g. TimeSpanParse.cs#L943-956)
fn finish(positive: bool, ticks: i64) -> Result<TimeSpan, TimeSpanError> {
    let ticks = if positive {
        ticks
    } else {
        let negated = ticks.wrapping_neg();
        if negated > 0 {
            return Err(TimeSpanError::Overflow);
        }
        negated
    };
    Ok(TimeSpan::from_ticks(ticks))
}
