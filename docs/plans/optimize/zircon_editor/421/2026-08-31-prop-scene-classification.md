---
title: Editor421 Prop Scene Classification
category: zircon_editor
report_id: Editor421-prop-scene-classification-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor421 Prop Scene Classification

Viewport prop scene classification now performs the `Cargo` substring search once before deciding
between `CargoInner` and `Cargo`. Exact prop IDs, Rack fallback, classification priority, and
unknown handling are unchanged.

Regression coverage verifies exact IDs, CargoInner/Cargo priority, Rack, and unknown values. The
ignored Windows Release benchmark emits `EDITOR421_PROP_SCENE_CLASSIFICATION_BENCH_V1` over
100,000 Cargo-only classifications. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.75` (at least 25% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor421 is prepared with Runtime491 under request
`runtime491-editor421-performance-batch-20260831gi-v1`. Receipt, validation ticket, measured
p95, pushed SHA, and notification result are recorded only after coordinator completion.
