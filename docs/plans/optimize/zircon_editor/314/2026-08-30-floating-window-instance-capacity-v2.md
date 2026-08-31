---
title: Editor314 Floating Window Instance Capacity v2
category: zircon_editor
report_id: Editor314-floating-window-instance-capacity-2026-08-30-v2
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor314 Floating Window Instance Capacity v2

This v2 record pairs with the corrected Runtime368 benchmark source. The Editor recursive count,
depth-first ordering, and exact-capacity behavior are unchanged from v1.

The ignored Windows Release benchmark emits `EDITOR314_FLOATING_WINDOW_INSTANCE_CAPACITY_BENCH_V1`
and requires `candidate_p95_ns <= baseline_p95_ns * 0.70`. The coordinator owns the combined
Runtime/Editor compile, six-test batch, exact p95 evidence, record finalization, manifest-only
commit/push, and one-shot WeCom publication. The v1 ticket remains immutable stale evidence.
