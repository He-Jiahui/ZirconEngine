---
title: Editor314 Floating Window Instance Capacity v3
category: zircon_editor
report_id: Editor314-floating-window-instance-capacity-2026-08-30-v3
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor314 Floating Window Instance Capacity v3

v3 constructs the split workspace fixture through the public Serde representation so the behavior
test does not access private `DocumentNode` fields. Production recursive counting and the capacity
benchmark are unchanged. The ignored Release benchmark emits
`EDITOR314_FLOATING_WINDOW_INSTANCE_CAPACITY_BENCH_V1` and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

The v1 and v2 tickets remain immutable stale evidence. The coordinator owns combined compile,
six-test execution, exact p95 evidence, record finalization, manifest-only commit/push, and one-shot
WeCom publication.

The v3 ticket `45733121275542308c77541d83c6ac78` was rejected as `snapshot_stale`; the current
pair is resubmitted under request `runtime368-editor314-performance-batch-20260830bp-v4`.
