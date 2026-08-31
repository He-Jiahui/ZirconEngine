---
title: Editor388 Direct Slider Percent Label
category: zircon_editor
report_id: Editor388-direct-slider-percent-label-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor388 Direct Slider Percent Label

Retained slider painting now encodes normalized two-decimal labels directly into one four-byte
string. Converting the clamped `f32` to `f64` is exact; multiplying by 100 remains exact for the
input precision, and `round_ties_even` preserves Rust's two-decimal formatting rule.

Custom value labels remain unchanged. Regression coverage checks normal values, half-cent rounding
boundaries and adjacent `f32` values, clamping, infinities, NaN, and negative zero against the
former formatter. Non-finite values and negative zero deliberately retain the standard formatter
fallback so their bytes remain unchanged.

The ignored Windows Release benchmark emits `EDITOR388_DIRECT_SLIDER_PERCENT_LABEL_BENCH_V1` over
17 alternating paired samples, each producing 262,144 normalized labels. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.70` (at least 30% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor388 is prepared with Runtime458 under request
`runtime458-editor388-performance-batch-20260831ez-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
