---
title: Runtime576 Mesh SDF Seed Validation Preflight
category: zircon_runtime
report_id: Runtime576-mesh-sdf-seed-validation-preflight-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime576 Mesh SDF Seed Validation Preflight

Runtime mesh preparation now validates every primitive Mesh SDF before cloning any payload into
the immutable seed. The previous interleaved loop cloned each valid payload immediately, so a
late invalid primitive discarded all earlier deep copies. The optimized failure path performs zero
payload clones while preserving missing-payload counts, invalid primitive indices, validation
errors, and the ready seed's primitive order.

Regression coverage verifies the late-invalid index and the validation-before-clone source shape.
The ignored Windows Release benchmark emits `RUNTIME576_MESH_SDF_SEED_PREFLIGHT_BENCH_V1`
across 31 alternating sample pairs, each running 100 preparations of 16 primitives with the final
payload invalid. This removes 15 deep copies of an approximately 8 KiB voxel buffer per
preparation. The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.90`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both batch gates pass.

## Current Batched Validation Handoff (2026-08-31)

Runtime576 is prepared with Editor576 under request
`runtime576-editor576-sdf-drag-performance-20260831gu-v1`. Receipt, validation ticket, measured
P95, pushed SHA, and notification result are recorded only after coordinator completion.
