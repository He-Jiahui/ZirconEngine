---
title: Runtime368 Capability Report Capacity
category: zircon_runtime
report_id: Runtime368-capability-report-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime368 Capability Report Capacity

## Scope

`RenderCapabilitySummary::capability_class_report` now reserves the fixed number of capability
slots for the requested class before partitioning satisfied and missing entries. The report keeps
the existing class filter, enum order, mismatch payload, and public result shape while avoiding
growth reallocations for the 11-entry experimental class.

## Tests And Performance Gate

The source file owns two non-ignored behavior/source-contract tests and one ignored Release
benchmark under the `optimization_batch_20260830bp_` prefix. The benchmark emits
`RUNTIME368_CAPABILITY_REPORT_CAPACITY_BENCH_V1`, compares the old growable vectors with the
class-capacity path over 250,000 reports per sample and 17 interleaved samples, and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo command was run. The coordinator owns the combined Runtime/Editor compile,
six-test batch, ignored Release benchmarks, exact p95 evidence, record finalization, manifest-only
commit, push to `origin/main`, and one-shot WeCom publication with the measured reduction.

## Batched validation handoff (2026-08-30)

Runtime368 is included with Editor314 under request `runtime-editor-368-314-20260830-v2`, ticket
`f4596d3ad8834de69ba3e5c54298e84c`, source manifest hash
`b6df7c1ede209c12488ce18e1360f5d58aec16760916472b220a3269f069f54c`, and the 30% p95 gate.
The manifest binds the compile-time validation resource
`zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/external_image_copy.rs` at
`a1102110c7daee234ea89c1f19491a267e64f7a0e4a2882fa61e1c1c47920606`.
