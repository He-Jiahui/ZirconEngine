---
title: Runtime447 Shared Container Gap Lookup
category: zircon_runtime
report_id: Runtime447-shared-container-gap-lookup-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime447 Shared Container Gap Lookup

Runtime UI template parsing now resolves FlowBox/FlexBox and GridBox/GridGroup axis-specific gaps
through one shared helper. When either axis falls back to the common `gap` property, that property is
looked up and parsed once, rather than once per missing axis. When both explicit axis values are
present, the common property is not read.

Horizontal/vertical and column/row precedence, invalid-value fallback, zero defaults, container
configuration, and all other layout parsing remain unchanged. Regression coverage requires both
container families to use the helper and bounds the common gap lookup to one occurrence.

The ignored Windows Release benchmark emits `RUNTIME447_SHARED_CONTAINER_GAP_LOOKUP_BENCH_V1` over
17 alternating paired samples. Each sample resolves 32,768 two-axis gaps from a 513-entry authored
table where both axes use the shared fallback. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.85`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime447 is prepared with Editor375 under request
`runtime447-editor375-performance-batch-20260831em-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
