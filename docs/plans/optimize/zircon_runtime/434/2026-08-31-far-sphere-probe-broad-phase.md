---
title: Runtime434 Far Sphere Probe Broad Phase
category: zircon_runtime
report_id: Runtime434-far-sphere-probe-broad-phase-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime434 Far Sphere Probe Broad Phase

Sphere reflection-probe influence now computes squared center distance once and rejects valid
positive-radius probes at or beyond their boundary before evaluating a square root. Scenes with
many distant probes therefore avoid the most expensive part of sphere influence evaluation during
per-position blend selection.

Boundary, blend-distance, and invalid-data behavior remain unchanged. Values inside the sphere
still evaluate the exact Euclidean distance, while nonpositive or nonfinite radii retain the legacy
fallback path. Regression tests cover the boundary, distant, and blended interior cases plus the
source-level broad-phase contract.

The ignored Windows Release benchmark emits
`RUNTIME434_FAR_SPHERE_PROBE_BROAD_PHASE_BENCH_V1` over 17 alternating paired samples. Each sample
performs 32 passes across 2,048 distant probes: the legacy path performs 65,536 square roots per
sample and the optimized broad phase performs none. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime434 is prepared with Editor362 under request
`runtime434-editor362-performance-batch-20260831dz-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
