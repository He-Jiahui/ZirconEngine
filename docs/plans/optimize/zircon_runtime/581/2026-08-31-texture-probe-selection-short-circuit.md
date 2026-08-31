---
title: Runtime581 Texture Probe Selection Short Circuit
category: zircon_runtime
report_id: Runtime581-texture-probe-selection-short-circuit-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime581 Texture Probe Selection Short Circuit

Texture-target planar-reflection selection now stops at the first matching probe that can produce
valid GPU parameters. That branch returns only a default feedback-suppression value, so the former
full scan and minimum-probe-ID reduction did not affect its result. Non-texture selection and its
deterministic minimum-ID policy remain unchanged.

Regression coverage verifies presence semantics for first, later, and absent valid candidates. The
ignored Windows Release benchmark emits `RUNTIME581_TEXTURE_PROBE_SELECTION_BENCH_V1` over 21
alternating sample pairs, 8,192 selections per sample, and 2,048 candidates with the first valid.
The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.50`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both batch gates pass.

## Current Batched Validation Handoff (2026-08-31)

Runtime581 is prepared with Editor581 under request
`runtime581-editor581-probe-badge-clip-performance-20260831gz-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
