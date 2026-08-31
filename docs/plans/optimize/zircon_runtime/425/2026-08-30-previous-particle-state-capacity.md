---
title: Runtime425 Previous Particle State Capacity
category: zircon_runtime
report_id: Runtime425-previous-particle-state-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime425 Previous Particle State Capacity

Successful particle submission now reserves the source sprite count after clearing the per-camera
previous-state vector. The existing anonymous-entity ambiguity filter and billboard snapshot
projection are unchanged; reserve is a no-op when the retained capacity already covers the frame.

The previous path could grow the retained vector geometrically when a camera first encountered a
large sprite batch or when the batch grew sharply. The new reserve bounds that growth to one
capacity decision per successful update.

The ignored Windows Release benchmark emits
`RUNTIME425_PREVIOUS_PARTICLE_STATE_CAPACITY_BENCH_V1` over 17 alternating paired samples, each
building 8,192 vectors of 512 sprite values, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime425 is prepared with Editor353 under request
`runtime425-editor353-performance-batch-20260830dq-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
