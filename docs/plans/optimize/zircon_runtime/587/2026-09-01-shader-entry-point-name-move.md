---
title: Runtime587 Shader Entry Point Name Move
category: zircon_runtime
report_id: Runtime587-shader-entry-point-name-move-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime587 Shader Entry Point Name Move

Shader IDE WGSL parsing and validation now consume the completed `naga::Module` when projecting
entry-point names. The former path borrowed a module that was discarded immediately afterward and
cloned every entry-point `String`; the new path moves those names into the validation result.

Parse diagnostics, validation diagnostics, entry-point order, and returned names remain unchanged.
Regression coverage retains valid/invalid WGSL behavior and requires the consuming source shape.

The ignored Windows Release benchmark emits `RUNTIME587_SHADER_ENTRY_POINT_MOVE_BENCH_V1` over 21
alternating sample pairs and 16,384 representative entry points per sample. The legacy model clones
16,384 qualified names per sample; the optimized model moves them without name allocation. The gate
requires `optimized_p95_ns <= legacy_p95_ns * 0.45`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime587 is prepared with Editor587 under request
`runtime587-editor587-shader-pane-performance-20260901hk-v1`. Receipt, validation ticket, measured
P95, pushed SHA, and notification result are recorded only after coordinator completion.
