---
title: Runtime401 Spatial Index Median Selection
category: zircon_runtime
report_id: Runtime401-spatial-index-select-nth-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime401 Spatial Index Median Selection

Navigation polygon spatial-index construction now selects only the median required by each BVH
split instead of fully sorting every recursive slice. Axis selection and the coordinate-plus-index
total comparator are unchanged, so the selected left/right polygon sets, bounds, leaf membership,
and nearest-query tie breaking remain equivalent while construction complexity drops from repeated
recursive sorting toward `O(n log n)`.

The ignored Windows Release benchmark emits `RUNTIME401_SPATIAL_INDEX_SELECT_NTH_BENCH_V1` over 15
alternating paired full-index builds with 32,768 polygons, requiring
`selected_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime401 is submitted with Runtime400 under request
`runtime400-runtime401-performance-batch-20260830cy-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
