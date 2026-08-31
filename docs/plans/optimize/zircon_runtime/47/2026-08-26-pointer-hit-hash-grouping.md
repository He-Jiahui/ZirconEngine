---
title: Runtime47 Pointer Hit Hash Grouping
category: zircon_runtime
report_id: Runtime47-pointer-hit-hash-grouping-2026-08-26
date: 2026-08-26
session_id: root-runtime47-three-task-picking-performance-batch-r2-20260831
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime47 Pointer Hit Hash Grouping

## Scope

This slice replaces the private pointer-hit grouping `BTreeMap` with `HashMap`. Each pointer group
still uses the existing priority, backend order, hit depth, output index, and hit index sort. The
finished groups are projected back into `BTreeMap`, so report construction, hover-map construction,
and public pointer iteration retain their previous deterministic order.

No hash iteration reaches an observable output. Repeated backend outputs for one pointer continue
to append hits in input order before the existing stable tie-breakers are applied.

## Performance Workload

The release workload groups 65,536 backend outputs across 4,096 pointers, with 16 hits per pointer.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered per-output group probes | 65,536 | 0 |
| Hash per-output group probes | 0 | 65,536 |
| Ordered final pointer projections | 0 | 4,096 |
| Per-pointer hit sorts | 4,096 | 4,096 |

The ignored release gate runs 21 alternating sample pairs and emits
`RUNTIME47_POINTER_HIT_HASH_GROUPING_BENCH_V1`. Acceptance requires the hash grouping plus ordered
boundary projection P95 to be at least 30% below the legacy direct `BTreeMap` grouping path. Exact
Windows P50/P95 timings remain pending the coordinator run.

## Acceptance

- `runtime47_batch_pointer_hash_groups_preserve_sorted_results` compares the
  complete projection with the legacy algorithm and locks ordered pointer keys.
- `runtime47_batch_pointer_grouping_keeps_hash_private_and_output_ordered`
  locks the private hash owner and explicit ordered boundary projection.
- `runtime47_batch_pointer_hash_grouping_release_benchmark` reports paired
  release P50/P95 samples and enforces the 30% P95 reduction gate.
- This task is queued in one Runtime47 three-task asynchronous validation batch with hover
  membership and pointer-location map. The batch runs nine `runtime47_batch_` Rust tests and three
  exact performance rows; no local Cargo lane is launched.

## Remaining Parent-plan Work

Runtime47 still owns GPU picking currentness, multi-camera routing, pointer lifecycle, backend
qualification, and product-scale latency. This slice only converges the CPU grouping index used by
the shared report and hover projection.
