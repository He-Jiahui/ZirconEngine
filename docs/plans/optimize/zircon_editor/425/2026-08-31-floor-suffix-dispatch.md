---
title: Editor425 Floor Suffix Dispatch
category: zircon_editor
report_id: Editor425-floor-suffix-dispatch-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor425 Floor Suffix Dispatch

Viewport floor classification now fast-paths canonical `WorkbenchViewport` suffix IDs before
falling back to legacy substring checks. FloorGrid, FloorPanel, FloorSeam, FloorGrate, embedded
token compatibility, and priority behavior remain unchanged.

Regression coverage verifies canonical suffix dispatch, an embedded-token fallback, and unknown
IDs. The ignored Windows Release benchmark emits `EDITOR425_FLOOR_SUFFIX_DISPATCH_BENCH_V1` over
100,000 canonical FloorGrate classifications. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.75` (at least 25% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor425 is prepared with Runtime495 under request
`runtime495-editor425-performance-batch-20260831gm-v1`. Receipt, validation ticket, measured
p95, pushed SHA, and notification result are recorded only after coordinator completion.
