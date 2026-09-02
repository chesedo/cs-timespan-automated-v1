//! Gungraun (instruction-count) benchmark for `TimeSpan::parse_exact`/`parse_exact_multiple`,
//! the custom-format-string parsing path (see issue #73).
//!
//! This is a structurally different algorithm from `parse_gungraun.rs`'s `Parse`/`from_str`
//! coverage: `Parse` (`src/time_span_parse.rs`) tokenizes the whole *input* up front into
//! number/separator tokens and dispatches by token count/shape to one of several pattern
//! handlers. `parse_exact` (`src/time_span_parse_exact.rs`) instead walks the *format* string
//! character-by-character against the input via a repeat-pattern/quote/escape tokenizer
//! (mirroring upstream's `TryParseByFormat`) — see that module's doc comment, and
//! `src/time_span_format_custom.rs`'s doc comment, which notes the two format-walking
//! tokenizers (this one reading, `format_customized`'s writing) "walk the format string
//! character-by-character via a repeat-pattern/quote/escape tokenizer with the same shape"
//! but in opposite directions. A benchmark covering only `Parse` would leave this whole
//! second tokenizer's performance uncharacterized and unprotected by the CI regression gate.
//!
//! `#[bench::single_format]` covers `parse_exact` with one format string that exercises
//! digit-matching (`dd`, `hh`, `mm`, `ss`), escape-handling (`\.`, `\:`), and fraction-matching
//! (`fffffff`) branches together in one pass.
//!
//! `#[bench::multiple_formats]` covers `parse_exact_multiple`, which retries each candidate
//! format in turn against the input until one matches — a cost `parse_exact` (single format)
//! never pays at all. The array below has three candidates, only the last of which actually
//! matches `INPUT`, so the benchmark also exercises the failed-attempt path for the first two
//! before the successful one.
//!
//! Discovered by `.github/workflows/bench.yml` via the `benches/*_gungraun.rs` naming
//! convention and run as a base-vs-head CI regression gate per the perf-verification skill:
//! deterministic instruction counts (via callgrind), no statistical noise smoothing needed. The
//! `soft_limits` below is the threshold above which a same-job base/head diff fails the gate.
use std::hint::black_box;

use cs_timespan_automated_v1::{TimeSpan, TimeSpanError, TimeSpanStyles};
use gungraun::prelude::*;
use gungraun::{Callgrind, EventKind};

/// `"01.02:03:04.5000000"` — a full days.hours:minutes:seconds.fraction value, chosen so a
/// single input drives every branch the format strings below exercise.
const INPUT: &str = "01.02:03:04.5000000";

/// Digit-matching (`dd`/`hh`/`mm`/`ss`), escape-handling (`\.`/`\:`), and fraction-matching
/// (`fffffff`) all in one format string.
const FORMAT: &str = r"dd\.hh\:mm\:ss\.fffffff";

#[library_benchmark]
#[bench::single_format(FORMAT)]
fn bench_parse_exact(format: &str) -> Result<TimeSpan, TimeSpanError> {
    black_box(TimeSpan::parse_exact(
        black_box(INPUT),
        black_box(format),
        TimeSpanStyles::None,
    ))
}

/// Three candidate formats, only the last of which matches `INPUT` — `parse_exact_multiple`
/// must fail the first two attempts before succeeding on the third.
const FORMATS: [&str; 3] = [r"hh\:mm\:ss", "%h", FORMAT];

#[library_benchmark]
#[bench::multiple_formats(&FORMATS)]
fn bench_parse_exact_multiple(formats: &[&str]) -> Result<TimeSpan, TimeSpanError> {
    black_box(TimeSpan::parse_exact_multiple(
        black_box(INPUT),
        black_box(formats),
        TimeSpanStyles::None,
    ))
}

library_benchmark_group!(
    name = parse_exact_group,
    benchmarks = [bench_parse_exact, bench_parse_exact_multiple]
);

main!(
    // 5% is comfortably above callgrind's own binary-layout noise floor (a few tenths of a
    // percent) while still catching a real regression in either the single-format or the
    // retry-loop path.
    config = LibraryBenchmarkConfig::default()
        .tool(Callgrind::default().soft_limits([(EventKind::Ir, 5.0)])),
    library_benchmark_groups = parse_exact_group
);
