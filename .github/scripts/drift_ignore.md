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
