---
title: Runtime432 Single Navigation Snapshot Serialization
category: zircon_runtime
report_id: Runtime432-single-navigation-snapshot-serialization-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime432 Single Navigation Snapshot Serialization

Navigation snapshot preparation now serializes the command once, clones that JSON value for the
result, and adds the nullable report field to the result object. It no longer deep-clones both
navigation snapshots into a second typed value and serializes the same large mesh data twice.

The command and result schemas are unchanged. Regression tests compare both optimized values with
the legacy typed serialization contract and enforce the single-serialization source path.

The ignored Windows Release benchmark emits
`RUNTIME432_SINGLE_NAVIGATION_SNAPSHOT_SERIALIZATION_BENCH_V1` over 17 alternating paired samples,
each preparing 16 changes containing 8,192-vertex navigation snapshots, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime432 is prepared with Editor360 under request
`runtime432-editor360-performance-batch-20260831dx-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
