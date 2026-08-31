---
title: Runtime381 Model Primitive Capacity
category: zircon_runtime
report_id: Runtime381-model-primitive-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime381 Model Primitive Capacity

GPU model construction and mesh-asset primitive selection now reserve exact primitive capacity
before populating their result vectors. Referenced mesh assets remain preferred, failed or absent
mesh conversions still fall back to the embedded primitive, and primitive order is unchanged.

The ignored Windows Release benchmark emits `RUNTIME381_MODEL_PRIMITIVE_CAPACITY_BENCH_V1` over
17 paired samples with 256 primitives per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime381 is submitted with Editor327 under request
`runtime381-editor327-performance-batch-20260830cc-v1`. Receipt, validation ticket, and source
manifest details are recorded in the session submission log after acceptance.
