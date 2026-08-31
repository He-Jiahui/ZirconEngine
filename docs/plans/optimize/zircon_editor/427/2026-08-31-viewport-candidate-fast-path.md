---
title: Editor427 Viewport Candidate Fast Path
category: zircon_editor
report_id: Editor427-viewport-candidate-fast-path-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor427 Viewport Candidate Fast Path

Viewport scene candidate filtering now rejects the known AxisLabel/Gizmo IDs before invoking the
broader chrome classifier. The WorkbenchViewport prefix requirement, chrome exclusion, axis/gizmo
exclusion, and unknown behavior remain unchanged.

Regression coverage verifies ordinary scene candidates, axis labels, gizmos, and non-viewport IDs.
The ignored Windows Release benchmark emits `EDITOR427_VIEWPORT_CANDIDATE_FAST_PATH_BENCH_V1`
over 100,000 known-gizmo classifications. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.75` (at least 25% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor427 is prepared with Runtime497 under request
`runtime497-editor427-performance-batch-20260831go-v1`. Receipt, validation ticket, measured
p95, pushed SHA, and notification result are recorded only after coordinator completion.
