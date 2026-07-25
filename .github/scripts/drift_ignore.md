# Drift-check ignore list

Categories of C#/Rust divergence that the drift-check scanner should stop
flagging, because they've been reviewed and judged intentional or not worth
replicating. Each entry describes a *category* of gap, not a single issue
title, so the scanner (an LLM reading this file) can recognize
similar-but-differently-worded future candidates in the same category.

When a `work-issue` run closes a drift-filed issue as out-of-scope, it adds
an entry here — see the work-issue skill's "Step 4b" for the process.

<!-- Example entry shape:

### Locale-specific decimal separator quirks

C#'s Globalization data has locale entries this port doesn't replicate
1:1 — intentional, since supporting every CLDR locale exactly as .NET does
is out of scope for this crate. Don't flag missing locale-specific
formatting behavior unless it affects the small fixed set of locales this
crate actually documents supporting.

-->

### Broad "add general unit tests" findings

Findings that claim the crate lacks unit test coverage in general terms —
e.g. "only one doctest exists", "no `#[cfg(test)] mod tests` anywhere",
or similar broad framing — rather than naming a specific untested
behavior. This crate's test suite (`tests/time_span.rs`) has grown
incrementally alongside each ported method via the drift-scan/work-issue
loop, so a snapshot claim like "X has zero coverage" goes stale within a
few issue cycles as more specific, narrower test-coverage issues land and
close first. Don't flag general/aggregate test-coverage gaps; instead let
the scanner keep filing (and this ignore list keep excluding) only
findings that name one specific method, operator, or branch that is
still genuinely untested as of the current `tests/time_span.rs` content.

### `i128`-widening "changes overflow detection" findings for multi-term sums

Findings claiming that widening a multi-term component sum to `i128` before
range-checking (e.g. `dhms_to_ticks` backing `from_dhms`/`from_dhms_milli`/
`from_dhms_micro`, or any similar helper that sums several `i32` components
each scaled by a tick-unit constant) is a divergence from C#, because C#'s
own sum is plain `long` arithmetic computed in an *unchecked* context and can
itself overflow `long`/wrap silently for sufficiently extreme component
magnitudes before its own range check runs.

This has been investigated (issue #58) for the 6-arg constructor
(TimeSpan.cs#L292-L306, backing `dhms_to_ticks`): confirmed empirically
(e.g. `days = 213_503_983` with the other components zero) that C#'s
unchecked `long` sum for `days * MicrosecondsPerDay` truly does overflow
`long` and wraps back into `[MinMicroseconds, MaxMicroseconds]`, so
`new TimeSpan(213503983, 0, 0, 0, 0, 0)` does *not* throw
`ArgumentOutOfRangeException` even though `days` is ~20x past `MaxDays`
(~10,675,199) — a silent, wrong result. Judged not worth replicating:

- The constructor's own XML doc unconditionally promises
  `ArgumentOutOfRangeException` for parameters outside `MinValue`/
  `MaxValue` — the wraparound contradicts C#'s own documented contract
  rather than expressing any intended semantic.
- `dotnet/runtime`'s own test suite
  (`System.Runtime.Tests/System/TimeSpanTests.cs`,
  `Ctor_Int_Int_Int_Int_Int_Int_Invalid` and siblings) only ever probes
  *just past* `MinValue`/`MaxValue` by one unit; it never approaches the
  magnitude needed to trigger `long` wraparound, so this isn't a pinned
  or regression-tested behavior on the C# side — it's an unexercised
  implementation artifact of unchecked arithmetic, not a spec.
- This crate already treats `i128` widening-then-range-check as the
  standard technique for these sums (see `time_to_ticks`'s doc comment),
  and reliably catching every genuinely out-of-range input is strictly
  more correct than C#'s implementation, not a gap to close.

Don't flag this pattern again for `dhms_to_ticks` or any structurally
similar multi-term-sum-then-range-check helper in this crate.
