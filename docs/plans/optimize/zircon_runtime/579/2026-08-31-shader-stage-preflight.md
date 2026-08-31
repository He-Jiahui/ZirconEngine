---
title: Runtime579 Shader Stage Preflight
category: zircon_runtime
report_id: Runtime579-shader-stage-preflight-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime579 Shader Stage Preflight

Shader entry-point projection now validates and resolves the authored stage before cloning the
entry-point name. Invalid stage tokens previously allocated and copied the complete name before the
`Option` short-circuit discarded it. Valid descriptors and all accepted stage aliases are unchanged,
while rejected shader metadata avoids that allocation entirely.

Regression coverage compares valid aliases and invalid stages with the legacy construction order.
The ignored Windows Release benchmark emits `RUNTIME579_SHADER_STAGE_PREFLIGHT_BENCH_V1` over 21
alternating sample pairs and 250,000 rejected descriptors per sample using a 1,600-byte entry name.
The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.50`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both batch gates pass.

## Current Batched Validation Handoff (2026-08-31)

Runtime579 is prepared with Editor579 under request
`runtime579-editor579-shader-origin-performance-20260831gx-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
