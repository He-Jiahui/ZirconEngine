---
title: Editor348 Single Allocation Builtin View Batches
category: zircon_editor
report_id: Editor348-single-allocation-builtin-view-batches-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor348 Single Allocation Builtin View Batches

Builtin activity-view and activity-window assembly now exposes the functional descriptor groups as
fixed arrays and chains them with the builtin arrays before collection. Exact iterator lengths let
each final descriptor vector allocate once while preserving all 22 activity views, all 16 activity
windows, and their established ordering.

The previous paths allocated the builtin vector and a separate functional vector, then grew the
builtin vector while extending it. Both registry batches now remove the functional temporary and
the destination growth, reducing their normal construction from about three heap allocations to
one.

The ignored Windows Release benchmark emits
`EDITOR348_SINGLE_ALLOCATION_BUILTIN_VIEW_BATCH_BENCH_V1` over 17 alternating paired samples,
each constructing 65,536 paired 12+10 and 9+7 batches, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Editor348 is prepared with Runtime420 under request
`runtime420-editor348-performance-batch-20260830dl-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
