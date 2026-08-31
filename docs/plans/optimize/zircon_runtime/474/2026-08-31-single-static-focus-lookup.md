---
title: Runtime474 Single Static Focus Lookup
category: zircon_runtime
report_id: Runtime474-single-static-focus-lookup-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime474 Single Static Focus Lookup

Runtime UI painter-state projection now resolves the static `focused` metadata attribute once and
reuses the result for both focused state and the legacy static focus-visible fallback. Component
focus and focus-visible precedence, static fixture behavior, and every other painter flag remain
unchanged.

This removes one ordered-map attribute lookup per painter-state projection on the static fallback
path. The source regression fixes one helper call and one underlying metadata read. The ignored
Windows Release benchmark emits `RUNTIME474_SINGLE_STATIC_FOCUS_LOOKUP_BENCH_V1` over 17
alternating paired samples and 262,144 resolutions per sample against representative metadata
containing 17 attributes. It reduces focused lookups per resolution from two to one. The gate
requires `optimized_p95_ns <= legacy_p95_ns * 0.65` (at least 35% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime474 is prepared with Editor404 under request
`runtime474-editor404-performance-batch-20260831fr-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
