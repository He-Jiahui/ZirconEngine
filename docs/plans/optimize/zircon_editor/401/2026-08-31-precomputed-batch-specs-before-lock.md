---
title: Editor401 Precomputed Batch Specs Before Lock
category: zircon_editor
report_id: Editor401-precomputed-batch-specs-before-lock-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor401 Precomputed Batch Specs Before Lock

Editor batch admission commit now builds its temporary job-spec reference table before acquiring
the scheduler state mutex. Reservation validation, job ID allocation, progress registration,
pending enqueue, and promotion order are unchanged.

The source contract locks the temporary allocation before the mutex acquisition. The ignored
Windows Release benchmark emits
`EDITOR401_PRECOMPUTED_BATCH_SPECS_LOCK_HOLD_BENCH_V1` over 17 alternating paired samples, 128 job
specs, and 4,096 isolated critical sections per sample. It records one legacy allocation under
the lock per collection versus zero after precomputation, while charging table release to both
critical sections. The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.75` (at least 25%
lower isolated lock-hold P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor401 is prepared with Runtime471 under request
`runtime471-editor401-performance-batch-20260831fo-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
