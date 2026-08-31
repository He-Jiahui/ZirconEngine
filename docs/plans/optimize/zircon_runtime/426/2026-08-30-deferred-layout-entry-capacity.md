---
title: Runtime426 Deferred Layout Entry Capacity
category: zircon_runtime
report_id: Runtime426-deferred-layout-entry-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime426 Deferred Layout Entry Capacity

Deferred lighting bind-group layout assembly now reserves the exact environment-only or full-
lighting entry count before appending the fixed provider arrays. The profile decision is computed
once, and all binding declarations, visibility, and provider ordering remain unchanged.

The previous path started with the five base entries and grew the vector as reflection, shadow,
lightmap, volumetric, cookie, irradiance, and runtime buffers were appended. Profile-aware capacity
eliminates those geometric growth steps during layout creation.

The ignored Windows Release benchmark emits
`RUNTIME426_DEFERRED_LAYOUT_ENTRY_CAPACITY_BENCH_V1` over 17 alternating paired samples, each
building 32,768 vectors for the 29-entry full-lighting profile, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime426 is prepared with Editor354 under request
`runtime426-editor354-performance-batch-20260830dr-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
