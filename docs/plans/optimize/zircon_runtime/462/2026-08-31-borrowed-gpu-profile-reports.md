---
title: Runtime462 Borrowed GPU Profile Reports
category: zircon_runtime
report_id: Runtime462-borrowed-gpu-profile-reports-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime462 Borrowed GPU Profile Reports

Render-frame statistics now pass the renderer's completed GPU timer and pipeline-statistics
reports directly to `FrameProfiler`. The per-frame update path no longer clones both diagnostic
report vectors merely to hand them to the profiler.

Report lifetimes remain bounded by the existing `write_frame_profile` call. Regression coverage
verifies pointer identity for both report types and preserves the `None` case.

The ignored Windows Release benchmark emits `RUNTIME462_BORROWED_GPU_PROFILE_REPORTS_BENCH_V1` over
17 alternating paired samples, each reading two 16-pass reports 16,384 times. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.10` (at least 90% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime462 is prepared with Editor392 under request
`runtime462-editor392-performance-batch-20260831fd-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
