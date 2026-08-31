---
title: Runtime451 Exact Color Strike Early Return
category: zircon_runtime
report_id: Runtime451-exact-color-strike-early-return-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime451 Exact Color Strike Early Return

Color bitmap strike selection now returns immediately after the first valid strike whose ppem
exactly matches the normalized target size. The former loop retained that same first exact strike
but continued validating and ranking every remaining bitmap strike even though none could replace
it.

Invalid-strike filtering, non-finite or non-positive target normalization, nearest-larger
downsampling, largest-smaller upscale fallback, first-equal ordering, scale, and raster metadata
remain unchanged. Regression coverage compares the optimized selector with the former full-scan
implementation for duplicate exact strikes and a normalized non-finite target.

The ignored Windows Release benchmark emits
`RUNTIME451_EXACT_COLOR_STRIKE_EARLY_RETURN_BENCH_V1` over 17 alternating paired samples. Each
sample performs 2,048 selections across 1,024 valid strikes with the exact strike first. The legacy
path scans all 1,024 strikes per selection while the optimized path examines one. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.10`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime451 is prepared with Editor381 under request
`runtime451-editor381-performance-batch-20260831es-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
