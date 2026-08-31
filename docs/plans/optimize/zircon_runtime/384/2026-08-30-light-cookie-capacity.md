---
title: Runtime384 Light Cookie Capacity
category: zircon_runtime
report_id: Runtime384-light-cookie-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime384 Light Cookie Capacity

Light-cookie atlas blit planning now reserves its draw and bind-group storage from the atlas entry
count before filtering unavailable textures. Resource checks, viewport calculation, entry order,
bind-group lifetime across the render pass, and draw count remain unchanged.

The ignored Windows Release benchmark emits `RUNTIME384_LIGHT_COOKIE_CAPACITY_BENCH_V1` over 17
paired samples with 512 entries per sample, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime384 is submitted with Editor330 under request
`runtime384-editor330-performance-batch-20260830cf-v1`. Receipt, validation ticket, and source
manifest details are recorded in the session submission log after acceptance.

Validation attempt: ticket `84553760a0ff42c6a0968c82b0723873` failed during coordinator
materialization with `unmanaged_artifacts_detected` for
`D:\ZirconBuilds\mvp-test-fixtures-36724`. Cargo did not start; regression, P95, commit/push, and
WeCom success evidence remain pending.
