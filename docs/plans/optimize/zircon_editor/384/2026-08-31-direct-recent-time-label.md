---
title: Editor384 Direct Recent Time Label
category: zircon_editor
report_id: Editor384-direct-recent-time-label-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor384 Direct Recent Time Label

The Editor recent-project snapshot now derives elapsed seconds once and writes minute, hour, and
day labels directly into an exact-capacity string. The former list projection constructed a
`Duration` and invoked the generic formatter for every dynamic label.

`Unknown`, `Just now`, saturating future timestamps, truncation at minute/hour/day boundaries, and
the exact `{count}{unit} ago` output remain unchanged. Regression coverage compares every boundary
family with the former implementation.

The ignored Windows Release benchmark emits `EDITOR384_DIRECT_RECENT_TIME_LABEL_BENCH_V1` over 17
alternating paired samples, each formatting 262,144 mixed minute, hour, and day labels. The
optimized path performs one decimal digit pass into one exact-capacity allocation. The gate
requires `optimized_p95_ns <= legacy_p95_ns * 0.75` (at least 25% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor384 is prepared with Runtime454 under request
`runtime454-editor384-performance-batch-20260831ev-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
