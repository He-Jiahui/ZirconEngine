---
title: Runtime406 OIT Compact Index Sort
category: zircon_runtime
report_id: Runtime406-oit-compact-index-sort-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime406 OIT Compact Index Sort

CPU OIT resolve now builds and unstable-sorts compact `(total-depth-key, source index)` entries
instead of cloning and stable-sorting every full fragment. The integer depth transform reproduces
`f32::total_cmp`, including signed zero, infinities, and NaNs; the source-index lane preserves the
previous stable order for equal depths. Blending and exact-depth output therefore traverse the same
fragment sequence without owning a second fragment array.

The ignored Windows Release benchmark emits `RUNTIME406_OIT_INDEX_SORT_BENCH_V1` over 17
alternating paired samples with 65,536 fragments and 4,096 exact layers, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime406 is submitted with Runtime407 under request
`runtime406-runtime407-performance-batch-20260830db-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
