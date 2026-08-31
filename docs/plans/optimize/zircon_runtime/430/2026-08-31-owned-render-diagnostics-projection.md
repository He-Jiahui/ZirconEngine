---
title: Runtime430 Owned Render Diagnostics Projection
category: zircon_runtime
report_id: Runtime430-owned-render-diagnostics-projection-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime430 Owned Render Diagnostics Projection

The dynamic runtime API now consumes the render statistics already owned by the collected
diagnostics snapshot. Backend and adapter strings move directly into the public diagnostics
response instead of allocating and copying the three strings before the source snapshot is
dropped.

The response schema, missing-render behavior, device-limit projection, and backend-name filtering
remain unchanged. A pointer-preservation regression test verifies that the response retains the
original string buffers.

The ignored Windows Release benchmark emits
`RUNTIME430_OWNED_RENDER_DIAGNOSTICS_PROJECTION_BENCH_V1` over 17 alternating paired samples, each
projecting 256 snapshots with three 4,096-byte strings, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gates.

## Current batched validation handoff (2026-08-31)

Runtime430 is prepared with Runtime429, Editor357, and Editor358 under request
`runtime429-430-editor357-358-performance-batch-20260831dv-v1`. This recovery batch retains the
`optimization_batch_du_` filter because the earlier request stopped at artifact governance before
compilation. Receipt, validation ticket, measured p95, pushed SHA, and notification result are
recorded only after coordinator completion.
