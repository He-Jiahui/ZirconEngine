---
title: Runtime382 Wireframe Capacity
category: zircon_runtime
report_id: Runtime382-wireframe-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime382 Wireframe Capacity

Per-frame wireframe generation now reserves the WireOnly highlight membership set from the exact
entity count and reserves two output vertices for every wire segment before emission. Shaded mode
still exits before allocation, display colors and selection behavior are unchanged, and mesh and
segment order remain stable.

The ignored Windows Release benchmark emits `RUNTIME382_WIREFRAME_CAPACITY_BENCH_V1` over 17
paired samples with 4,096 entities and 32 meshes of 256 segments, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime382 is submitted with Editor328 under request
`runtime382-editor328-performance-batch-20260830cd-v1`. Receipt, validation ticket, and source
manifest details are recorded in the session submission log after acceptance.
