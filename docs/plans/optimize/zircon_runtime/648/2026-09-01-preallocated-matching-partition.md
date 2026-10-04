---
title: Runtime648 Preallocated Matching Queue Partition
category: zircon_runtime
report_id: Runtime648-preallocated-matching-partition-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime648 Preallocated Matching Queue Partition

Keyed-I/O queue partitioning now reserves the matching output vector from the pending queue length.
The matching set is a subset of pending entries, so this is a strict upper bound; retained entries
still move back in original order and matched entries retain their prior order.

The ignored Windows Release benchmark emits `RUNTIME648_MATCHING_PARTITION_CAPACITY_BENCH_V1` over
17 alternating sample pairs with 65,536 queue entries and a 50% match rate. The gate requires
reserved P95 to be at most 80% of unreserved P95.

No direct Cargo validation was run. The coordinator owns combined Runtime648/Editor648 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime648 is prepared with Editor648 under the shared `optimization_batch_ji_` prefix. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.
