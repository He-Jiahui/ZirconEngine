---
title: Editor429 Viewport Unit Suffix Dispatch
category: zircon_editor
report_id: Editor429-viewport-unit-suffix-dispatch-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor429 Viewport Unit Suffix Dispatch

CSS-like dimension parsing now dispatches unsupported viewport units from the final byte before
checking the matching suffix. This replaces four sequential suffix scans while preserving the
exact `vw`, `vh`, `vmin`, and `vmax` diagnostics and the supported pixel/percentage paths.

Regression coverage verifies all four unsupported units and false-positive boundaries. The
ignored Windows Release benchmark emits `EDITOR429_VIEWPORT_UNIT_SUFFIX_DISPATCH_BENCH_V1`
across 31 samples of 100,000 worst-case `vmax` parses. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.90` (at least 10% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-08-31)

Editor429 is prepared with Runtime499 under request
`runtime499-editor429-performance-batch-20260831gq-v1`. Receipt, validation ticket, measured
P95, pushed SHA, and notification result are recorded only after coordinator completion.
