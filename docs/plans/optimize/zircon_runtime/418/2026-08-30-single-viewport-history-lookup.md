---
title: Runtime418 Single Viewport History Lookup
category: zircon_runtime
report_id: Runtime418-single-viewport-history-lookup-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime418 Single Viewport History Lookup

Viewport submission-state resolution now looks up the current camera history once and projects its
visibility, static index, and dynamic index from the retained reference. The existing snapshot
clones remain inside the state lock, so ownership and lifetime boundaries are unchanged.

The previous path hashed and compared the same `ViewportCameraHistoryKey` three times per viewport
and frame. The new path performs one map lookup while preserving missing-history behavior and the
three independently owned outputs.

The ignored Windows Release benchmark emits
`RUNTIME418_SINGLE_VIEWPORT_HISTORY_LOOKUP_BENCH_V1` over 17 alternating paired samples, each
projecting 65,536 keys across 4,096 retained histories, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime418 is prepared with Editor346 under request
`runtime418-editor346-performance-batch-20260830dj-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
