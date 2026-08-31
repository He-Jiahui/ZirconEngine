---
title: Editor347 Reserved Breadcrumb Pair
category: zircon_editor
report_id: Editor347-reserved-breadcrumb-pair-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor347 Reserved Breadcrumb Pair

Workbench and exclusive-page breadcrumb projection now share a two-slot buffer constructor. The
root page label is inserted into a vector already sized for the optional or required second label,
without changing active-view, welcome-page, serialized-path, or fallback-title selection.

The previous path created a one-element vector with capacity one and then pushed a second element
for the normal two-level breadcrumb, forcing an allocation growth and moving the first model.
The new path performs one vector allocation for that pair.

The ignored Windows Release benchmark emits
`EDITOR347_RESERVED_BREADCRUMB_PAIR_BENCH_V1` over 17 alternating paired samples, each constructing
65,536 two-element buffers, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Editor347 is prepared with Runtime419 under request
`runtime419-editor347-performance-batch-20260830dk-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
