---
title: Editor322 Close Instance Capacity
category: zircon_editor
report_id: Editor322-close-instance-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor322 Close Instance Capacity

`resolve_floating_window_close_instances` now reserves the tab count before cloning closeable tab
instance IDs. Empty/non-closeable rejection, cloning behavior, and tab order are unchanged.

The ignored Windows Release benchmark emits `EDITOR322_CLOSE_INSTANCE_CAPACITY_BENCH_V1` over 17
paired samples with 64 tabs per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.
No direct Cargo validation was run; the coordinator owns combined Release validation, records,
manifest-only commit/push, and one-shot WeCom publication after measured evidence passes.

## Current batched validation handoff (2026-08-30)

Editor322 is submitted in the eight-task batch under request
`runtime375-378-editor321-324-performance-batch-20260830-v1`. Receipt, ticket, and manifest details
are recorded in the submission log after acceptance.
