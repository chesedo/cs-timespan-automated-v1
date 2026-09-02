//! Gungraun (instruction-count) benchmark for `TimeSpan::from_str`/`Parse`, the crate's most
//! realistic hot path (log ingestion, CSV/config parsing, deserialization — see issue #70).
//!
//! Each `#[bench::...]` case is a distinct input shape chosen to hit a different dispatch
//! branch in `src/time_span_parse.rs`'s tokenizer (see that module's doc comment) — a single
//! input wouldn't catch a regression isolated to one branch.
use std::hint::black_box;

use cs_timespan_automated_v1::{TimeSpan, TimeSpanError};
use gungraun::prelude::*;
use gungraun::{Callgrind, EventKind};

#[library_benchmark]
// Minimal: a single integer -> `process_d` (days only).
#[bench::single_integer("3")]
// Bare-days boundary value at TimeSpan::MAX's day count -> `process_d`, but at the
// realistic upper bound rather than the trivial `single_integer` case.
#[bench::max_days_boundary("10675199")]
// Two components -> `process_hm` (hours:minutes).
#[bench::two_components("02:03")]
// Three components -> `process_hm_s_d` (hours:minutes:seconds, or days:hours:minutes).
#[bench::three_components("02:03:04")]
// Four components plus a fraction -> `process_hms_f_d`.
#[bench::four_components_with_fraction("02:03:04.5000000")]
// Full five-component "d.h:m:s.f" form -> `process_dhmsf`.
#[bench::full_five_component("1.02:03:04.5000000")]
// Same full form, negated, to exercise the sign-handling path separately.
#[bench::negative_five_component("-1.02:03:04.5000000")]
// Colon-separated day component (":" is unconditionally a valid day/hour separator for
// lenient Parse, not just the invariant-specific "." used above) -> exercises the
// `is_day_hour_sep` branch the other five-component cases don't.
#[bench::day_colon_separator("6:12:14:45.348")]
// TimeSpan::MIN's exact Display/"c"-format string, round-tripped back through lenient
// Parse -> the one value whose negation takes the documented i128-widening special path.
#[bench::min_boundary("-10675199.02:48:05.4775808")]
fn bench_parse(input: &str) -> Result<TimeSpan, TimeSpanError> {
    black_box(black_box(input).parse::<TimeSpan>())
}

library_benchmark_group!(name = parse_group, benchmarks = bench_parse);

main!(
    // 5% is comfortably above callgrind's own binary-layout noise floor (a few tenths of a
    // percent) while still catching a real regression in any one shape's dispatch branch.
    config = LibraryBenchmarkConfig::default()
        .tool(Callgrind::default().soft_limits([(EventKind::Ir, 5.0)])),
    library_benchmark_groups = parse_group
);
