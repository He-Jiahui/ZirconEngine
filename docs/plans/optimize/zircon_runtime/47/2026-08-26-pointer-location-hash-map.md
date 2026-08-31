---
title: Runtime47 Pointer Location Hash Map
category: zircon_runtime
report_id: Runtime47-pointer-location-hash-map-2026-08-26
date: 2026-08-26
session_id: root-runtime47-three-task-picking-performance-batch-r2-20260831
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime47 Pointer Location Hash Map

## Scope

This slice reduces per-frame pointer-event CPU work when current locations and input overrides are
indexed for hover transition dispatch. It supports Runtime47's pointer interaction state and the
PERF-MVP-332 event-amplification qualification. It does not claim product frame wiring, capture,
multi-view identity, drag thresholds, backend qualification, or an event budget.

## Change

- Replace the temporary ordered pointer-location map with a capacity-sized `HashMap`.
- Reserve the upper bound of location snapshots plus input records before insertion.
- Keep input records as the final writer for a pointer ID, preserving the existing override
  behavior.
- Change only the two lookup-only dispatch consumers; persistent ordered button/target state
  remains on its existing `BTreeMap` contract.

## Deterministic Performance Evidence

| 32,768 locations + 16,384 input overrides | Before | After | Reduction |
|---|---:|---:|---:|
| Ordered-tree writes | 49,152 | 0 | 100% removed |
| Hash-index writes | 0 | 49,152 | lookup index introduced |
| Growth planning | incremental tree nodes | one upper-bound reservation | bounded |
| Input override semantics | last input wins | last input wins | unchanged |

The ignored release gate warms both paths, then alternates 17 ordered-map and capacity-hash sample
pairs (9 legacy-first and 8 hash-first). It emits
`RUNTIME47_POINTER_LOCATION_HASH_MAP_BENCH_V1`; acceptance requires hash P95 to be at most 60% of
legacy P95. Exact Windows timings remain pending the batched coordinator run.

## Acceptance

- `runtime47_batch_location_map_preserves_input_override` covers duplicate
  pointer replacement and unrelated pointer retention.
- `runtime47_batch_location_map_uses_capacity_hash_index` requires the
  capacity-sized hash index and both lookup consumer signatures.
- `runtime47_batch_pointer_location_hash_map_performance_evidence` emits
  both P95 values and enforces the 60% threshold.
- This task is queued in one Runtime47 three-task asynchronous validation batch with hover
  membership and pointer-hit grouping. The batch runs nine `runtime47_batch_` Rust tests and three
  exact performance rows; no local Cargo lane is launched.

## Remaining Parent-plan Work

Runtime47 still lacks product frame authority, qualified view/camera/pointer generations, capture,
drag/click policy, target invalidation, backend failures, UI-first arbitration, and end-to-end
multi-pointer/multi-view scale evidence.
