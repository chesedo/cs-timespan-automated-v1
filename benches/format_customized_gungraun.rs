//! Gungraun (instruction-count) benchmark for the custom-format-string path of
//! `TimeSpan::to_string_format`/`TimeSpan::try_format` (see issue #72).
//!
//! Both entry points funnel any non-standard format string into
//! `format_customized` (`src/time_span_format_custom.rs`), which builds its output in
//! an unsized `String::new()` (no `.with_capacity` sizing hint) — a plausible future
//! optimization target once this benchmark gives it something to measure against (see
//! that function's `pub(crate) fn format_customized` doc comment). `try_format` in
//! particular documents that it does *not* extend `try_format`'s non-allocating
//! guarantee to custom formats: it routes through `format_customized` (one
//! intermediate `String` allocation) and then copies those bytes into the caller's
//! buffer — "a deliberate tradeoff rather than an oversight" per `try_format`'s doc
//! comment. This file benchmarks both entry points so a future optimization to
//! `format_customized` (e.g. `String::with_capacity`) shows up as a measured
//! improvement on both, not just one.
//!
//! The format string used, `"dd\.hh\:mm\:ss\.fffffff"`, is chosen to exercise every
//! token kind `format_customized`'s tokenizer dispatches on in a single call: `'d'`
//! (day digits), `'h'`/`'m'`/`'s'` (digit-formatting via `format_digits`), `'f'`
//! (fraction-formatting via `pow10_up_to_max_fraction_digits`), and `'\\'` (the escape
//! branch, for the literal `.`/`:` separators) — see the `match` arms in
//! `format_customized`, `src/time_span_format_custom.rs`.
//!
//! Discovered by `.github/workflows/bench.yml` via the `benches/*_gungraun.rs` naming
//! convention and run as a base-vs-head CI regression gate per the perf-verification
//! skill: deterministic instruction counts (via callgrind), no statistical noise
//! smoothing needed. The `soft_limits` below is the threshold above which a same-job
//! base/head diff fails the gate.
use std::hint::black_box;

use cs_timespan_automated_v1::{TimeSpan, TimeSpanError};
use gungraun::prelude::*;
use gungraun::{Callgrind, EventKind};

/// Exercises 'd', 'h', 'm', 's', 'f', and '\\' (escaped literal separators) all in one
/// format string — see this file's module doc comment.
const CUSTOM_FORMAT: &str = "dd\\.hh\\:mm\\:ss\\.fffffff";

#[library_benchmark]
#[bench::custom_format(TimeSpan::from_hms(2, 3, 4).unwrap())]
fn bench_to_string_format(ts: TimeSpan) -> Result<String, TimeSpanError> {
    black_box(black_box(ts).to_string_format(black_box(CUSTOM_FORMAT)))
}

#[library_benchmark]
#[bench::custom_format(TimeSpan::from_hms(2, 3, 4).unwrap())]
fn bench_try_format(ts: TimeSpan) -> Result<usize, TimeSpanError> {
    // Buffer sized generously for "02.03:04.0000000" (well under 32 bytes) so the
    // benchmark measures `format_customized`'s allocation-then-copy cost, not
    // `InsufficientBuffer` bailout.
    let mut buf = [0u8; 32];
    black_box(black_box(ts).try_format(black_box(&mut buf), black_box(CUSTOM_FORMAT)))
}

library_benchmark_group!(
    name = format_customized_group,
    benchmarks = [bench_to_string_format, bench_try_format]
);

main!(
    // 5% is comfortably above callgrind's own binary-layout noise floor (a few tenths
    // of a percent) while still catching a real regression in the custom-format
    // tokenizer or the String-growth path it exercises.
    config = LibraryBenchmarkConfig::default()
        .tool(Callgrind::default().soft_limits([(EventKind::Ir, 5.0)])),
    library_benchmark_groups = format_customized_group
);
