---
title: Editor419 Matching Length Prefilter
category: zircon_editor
report_id: Editor419-matching-length-prefilter-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor419 Matching Length Prefilter

Inspector row kind matching now rejects candidates with a different byte length before running
case-insensitive comparison. Candidate order, ASCII case-insensitive behavior, and the empty
candidate behavior are unchanged.

Regression coverage verifies case-insensitive matches and length-mismatched non-matches. The
ignored Windows Release benchmark emits `EDITOR419_MATCHING_LENGTH_PREFILTER_BENCH_V1` over
100,000 repeated negative lookups. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.90` (at least 10% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor419 is prepared with Runtime489 under request
`runtime489-editor419-performance-batch-20260831gg-v1`. Receipt, validation ticket, measured
p95, pushed SHA, and notification result are recorded only after coordinator completion.
