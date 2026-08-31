---
title: Editor399 Borrowed Paint-Only View Invalidation
category: zircon_editor
report_id: Editor399-borrowed-paint-only-view-invalidation-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor399 Borrowed Paint-Only View Invalidation

View-scoped editor invalidation now borrows its `ViewInstanceId` and clones it only when the mask
requires a retained-host recompute scope. Paint-only, pointer-hover, viewport-image, render-only,
and empty-mask paths avoid allocating and copying the view ID while retaining the existing request
counters and dirty-domain behavior.

Regression coverage verifies that the combined paint-only mask increments diagnostics without
adding a recompute transaction. The ignored Windows Release benchmark emits
`EDITOR399_BORROWED_PAINT_ONLY_VIEW_BENCH_V1` over 17 alternating paired samples, each issuing
131,072 view-scoped paint-only invalidations. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.75` (at least 25% lower P95) and records the elimination of
131,072 heap allocations, string copies, and frees per sample.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor399 is prepared with Runtime469 under request
`runtime469-editor399-performance-batch-20260831fm-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
