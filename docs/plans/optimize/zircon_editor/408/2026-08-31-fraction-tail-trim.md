---
title: Editor408 Fraction Tail Trim
category: zircon_editor
report_id: Editor408-fraction-tail-trim-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor408 Fraction Tail Trim

Editor template compatibility snapshots now trim fixed-precision fractional zeros using only
tail checks. The fractional formatting branch previously rescanned the complete rendered string
for a decimal point before every removed zero even though finite values are formatted with three
fractional digits; non-finite labels do not end in zero. Integer, fractional, infinity, and NaN
rendering remain unchanged.

Regression coverage verifies long fractional text and representative finite and non-finite
values. The ignored Windows Release benchmark emits `EDITOR408_FRACTION_TAIL_TRIM_BENCH_V1` over
17 alternating paired samples and 131,072 trims per sample. For a `.200` suffix, the legacy path
performs three complete-string scans while the optimized path performs none. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.60` (at least 40% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor408 is prepared with Runtime478 under request
`runtime478-editor408-performance-batch-20260831fv-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
