---
title: Runtime446 Move Unresolved Feature Ids
category: zircon_runtime
report_id: Runtime446-move-unresolved-feature-ids-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime446 Move Unresolved Feature Ids

Runtime plugin feature blocking now moves each pending selection's `definition_key` into the
unresolved-id set with `mem::take`. The key was previously cloned even though the remaining report
path consumes only the pending active feature and status, so the original allocation was dropped
unused immediately after the blocking pass.

The unresolved set remains an owned `HashSet<String>`, capability-provider lookup remains unchanged,
and pending features retain their original iteration and report order. Regression coverage requires
the mutable pending projection and direct key move while rejecting restoration of the deep clone.

The ignored Windows Release benchmark emits `RUNTIME446_MOVE_UNRESOLVED_FEATURE_IDS_BENCH_V1` over
17 alternating paired samples. Each sample projects 4,096 long feature ids into the same owned hash
set; input preparation is excluded from timing. The legacy model performs 4,096 key allocations and
the optimized model performs none. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.75`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime446 is prepared with Editor374 under request
`runtime446-editor374-performance-batch-20260831el-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
