//! Gungraun (instruction-count) benchmark pinning `TimeSpan::try_format`'s documented
//! non-allocating claim for "c"/"g"/"G", against `to_string_format`'s always-allocating
//! path for the same formats — see issue #71 and `try_format`'s doc comment
//! (`src/time_span.rs`) for the full rationale.
//!
//! `bench_try_format` uses a much tighter soft limit than `bench_to_string_format` so an
//! accidental allocation creeping into `try_format_standard` shows up as a large,
//! easily-caught relative jump rather than being absorbed into a loose threshold.
//!
//! The sample value (1d 2h 3m 4.5s) gives every format non-trivial output, including
//! the day segment and, for "c"/"G", the fractional-seconds segment.
use std::hint::black_box;

use cs_timespan_automated_v1::TimeSpan;
use gungraun::prelude::*;
use gungraun::{Callgrind, EventKind, LibraryBenchmarkConfig};

/// 1 day, 2 hours, 3 minutes, 4.5 seconds — exercises the day segment and, for "c"/"G",
/// the fractional-seconds segment, in every case below.
fn sample() -> TimeSpan {
    TimeSpan::from_dhms_milli(1, 2, 3, 4, 500).unwrap()
}

#[library_benchmark(config = LibraryBenchmarkConfig::default()
    // Tight: `try_format_standard` writes a fixed handful of ASCII digits directly into
    // `destination` with no allocation, so its instruction count should barely move run
    // to run — a threshold this close to callgrind's own noise floor (a few tenths of a
    // percent) still catches even a single introduced malloc/free pair.
    .tool(Callgrind::default().soft_limits([(EventKind::Ir, 2.0)])))]
#[bench::c("c")]
#[bench::g("g")]
#[bench::g_upper("G")]
fn bench_try_format(format: &str) -> usize {
    let ts = black_box(sample());
    let mut buf = [0u8; 32];
    black_box(
        ts.try_format(black_box(&mut buf), black_box(format))
            .unwrap(),
    )
}

#[library_benchmark(config = LibraryBenchmarkConfig::default()
    // Same 5% used by benches/parse_gungraun.rs: `to_string_format` always allocates a
    // `String`, so this threshold just guards against *this* path regressing further,
    // not against it losing an allocation-free property it never claimed to have.
    .tool(Callgrind::default().soft_limits([(EventKind::Ir, 5.0)])))]
#[bench::c("c")]
#[bench::g("g")]
#[bench::g_upper("G")]
fn bench_to_string_format(format: &str) -> String {
    let ts = black_box(sample());
    black_box(ts.to_string_format(black_box(format)).unwrap())
}

// `bench_try_format`/`bench_to_string_format` above vary the *format* on one fixed,
// small, ordinary value. These two vary the *value* instead, at a fixed format ("c",
// the `Display`/default format), covering the zero value, a value with no day
// component, a negative value with a day and fraction, and `TimeSpan::MIN`/`MAX` — the
// one case where `Display`/`format_general` take a documented special path: the doc
// comment directly above `impl std::fmt::Display for TimeSpan` in `src/time_span.rs`
// explains that `ticks` is widened to `i128` before negating specifically because
// `-i64::MIN` overflows `i64`. No other benchmark in this crate exercises that
// widening path.
//
// `TimeSpan::MIN`'s "c" output, `"-10675199.02:48:05.4775808"`, is 26 bytes — it fits
// the existing 32-byte buffer used above without any buffer-size change.
#[library_benchmark(config = LibraryBenchmarkConfig::default()
    .tool(Callgrind::default().soft_limits([(EventKind::Ir, 2.0)])))]
#[bench::zero(TimeSpan::ZERO)]
#[bench::no_days(TimeSpan::from_hms(2, 3, 4).unwrap())]
#[bench::negative(-TimeSpan::from_dhms_milli(1, 2, 3, 4, 500).unwrap())]
#[bench::min(TimeSpan::MIN)]
#[bench::max(TimeSpan::MAX)]
fn bench_try_format_by_value(ts: TimeSpan) -> usize {
    let mut buf = [0u8; 32];
    black_box(black_box(ts).try_format(black_box(&mut buf), "c").unwrap())
}

#[library_benchmark(config = LibraryBenchmarkConfig::default()
    .tool(Callgrind::default().soft_limits([(EventKind::Ir, 5.0)])))]
#[bench::zero(TimeSpan::ZERO)]
#[bench::no_days(TimeSpan::from_hms(2, 3, 4).unwrap())]
#[bench::negative(-TimeSpan::from_dhms_milli(1, 2, 3, 4, 500).unwrap())]
#[bench::min(TimeSpan::MIN)]
#[bench::max(TimeSpan::MAX)]
fn bench_to_string_format_by_value(ts: TimeSpan) -> String {
    black_box(black_box(ts).to_string_format("c").unwrap())
}

library_benchmark_group!(
    name = try_format_group,
    benchmarks = [bench_try_format, bench_try_format_by_value]
);
library_benchmark_group!(
    name = to_string_format_group,
    benchmarks = [bench_to_string_format, bench_to_string_format_by_value]
);

main!(
    library_benchmark_groups = try_format_group,
    to_string_format_group
);
