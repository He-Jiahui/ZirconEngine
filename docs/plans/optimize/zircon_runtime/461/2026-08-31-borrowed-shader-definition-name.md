---
title: Runtime461 Borrowed Shader Definition Name
category: zircon_runtime
report_id: Runtime461-borrowed-shader-definition-name-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime461 Borrowed Shader Definition Name

Shader definition name normalization now returns the trimmed slice borrowed from the definition's
owned name. IDE validation source generation and runtime shader-template assembly no longer allocate
and copy an otherwise read-only name before formatting the final WGSL constant.

Whitespace trimming, empty names, and the original owned name remain unchanged. Regression coverage
also verifies that a normalized nonempty slice points into the original allocation.

The ignored Windows Release benchmark emits
`RUNTIME461_BORROWED_SHADER_DEFINITION_NAME_BENCH_V1` over 17 alternating paired samples, each
normalizing 1,048,576 representative names. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.30` (at least 70% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime461 is prepared with Editor391 under request
`runtime461-editor391-performance-batch-20260831fc-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
