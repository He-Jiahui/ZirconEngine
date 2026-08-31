---
title: Runtime472 Borrowed Slider Value Key Update
category: zircon_runtime
report_id: Runtime472-borrowed-slider-value-key-update-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime472 Borrowed Slider Value Key Update

Range-slider synchronization now looks up an existing component-state value with the borrowed
property name and updates the value in place. A missing property still allocates and inserts its
owned key, and every update still clears drag/drop reference provenance before changing the value.
Slider clamping, percent conversion, descriptor fallback, and recursive value/percent sync are
unchanged.

This removes a temporary property-key allocation from each steady-state slider update. Regression
coverage checks both existing-key replacement and first insertion. The ignored Windows Release
benchmark emits `RUNTIME472_BORROWED_SLIDER_VALUE_KEY_BENCH_V1` over 17 alternating paired samples
and 65,536 existing-key updates per sample. The pressure case reduces owned key copies per sample
from 65,536 to zero. The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.70` (at least 30%
lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime472 is prepared with Editor402 under request
`runtime472-editor402-performance-batch-20260831fp-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
