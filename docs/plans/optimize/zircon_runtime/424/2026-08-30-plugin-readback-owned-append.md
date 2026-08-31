---
title: Runtime424 Plugin Readback Owned Append
category: zircon_runtime
report_id: Runtime424-plugin-readback-owned-append-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime424 Plugin Readback Owned Append

Plugin renderer readback merging now transfers every owned virtual-geometry and Hybrid-GI sideband
allocation with `Vec::append`. Existing empty-base adoption, renderer ordering, scalar max/sum
aggregation, global-SDF replacement, and particle single-owner behavior are unchanged.

The previous implementation extended each destination vector element by element after destructuring
the incoming output. The new path moves the incoming vectors into the destination with no per-item
copy on the merge path.

The ignored Windows Release benchmark emits
`RUNTIME424_PLUGIN_READBACK_OWNED_APPEND_BENCH_V1` over 17 alternating paired samples, each with
4,096 merges of 1,024 owned values, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime424 is prepared with Editor352 under request
`runtime424-editor352-performance-batch-20260830dp-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
