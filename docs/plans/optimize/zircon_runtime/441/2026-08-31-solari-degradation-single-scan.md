---
title: Runtime441 Solari Degradation Single Scan
category: zircon_runtime
report_id: Runtime441-solari-degradation-single-scan-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime441 Solari Degradation Single Scan

Per-frame Solari diagnostic recording now classifies all degradation reasons in one traversal. The
former recorder independently filtered the same degradation slice four times before publishing the
four reason counters.

The total degradation count, every reason-specific metric name, metric tags, frame index, and
counter values remain unchanged. Regression coverage exercises all four exhaustive enum variants,
including duplicate reasons, and requires the recorder to use the shared single-pass classifier.

The ignored Windows Release benchmark emits `RUNTIME441_SOLARI_DEGRADATION_SINGLE_SCAN_BENCH_V1`
over 17 alternating paired samples. Each sample classifies 8,192 reasons 256 times. The legacy
model performs four full slice passes per classification; the optimized model performs one. The
gate requires `optimized_p95_ns <= legacy_p95_ns * 0.60`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime441 is prepared with Editor369 under request
`runtime441-editor369-performance-batch-20260831eg-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
