---
title: Editor390 Direct Invalidation Stats
category: zircon_editor
report_id: Editor390-direct-invalidation-stats-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor390 Direct Invalidation Stats

Retained-host invalidation diagnostics now write nine counters directly into one preallocated
summary string. The fixed field names and stack-resident decimal buffers replace generic formatting
when verbose paint, slow-path, or render-path diagnostics are emitted.

Field order, separators, names, and decimal bytes remain unchanged from zero through `u64::MAX`.
Regression coverage compares zero, ordinary counters, decimal-width transitions, and maximum
values with the former formatter.

The ignored Windows Release benchmark emits `EDITOR390_DIRECT_INVALIDATION_STATS_BENCH_V1` over 17
alternating paired samples, each producing 131,072 nine-counter summaries. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.75` (at least 25% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor390 is prepared with Runtime460 under request
`runtime460-editor390-performance-batch-20260831fb-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
