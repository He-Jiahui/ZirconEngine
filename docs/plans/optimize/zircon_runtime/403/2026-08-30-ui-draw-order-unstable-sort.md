---
title: Runtime403 UI Draw Order In-Place Sort
category: zircon_runtime
report_id: Runtime403-ui-draw-order-unstable-sort-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime403 UI Draw Order In-Place Sort

UI-tree draw-order projection now sorts in place with a total key of z-index, paint order, and node
ID. The node-ID lane preserves the BTree node iteration order that the previous stable sort used for
legacy duplicate paint-order data, while normal monotonic paint-order trees avoid stable-sort scratch
allocation on each projection.

The ignored Windows Release benchmark emits `RUNTIME403_UI_DRAW_ORDER_UNSTABLE_SORT_BENCH_V1` over
17 alternating paired samples with 65,536 retained nodes, requiring
`unstable_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime403 is submitted with Runtime402 under request
`runtime402-runtime403-performance-batch-20260830cz-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
