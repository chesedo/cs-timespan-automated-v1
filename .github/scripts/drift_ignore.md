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
