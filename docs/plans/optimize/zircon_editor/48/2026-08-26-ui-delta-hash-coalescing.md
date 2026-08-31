---
title: Editor48 UI Delta Hash Coalescing
category: zircon_editor
report_id: Editor48-ui-delta-hash-coalescing-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor48 UI Delta Hash Coalescing

## Scope

This slice replaces the per-segment UI reflection-patch coalescing map with `HashMap`. Every patch
now reaches its latest-wins node owner through expected constant-time lookup while continuous
properties and pressed state keep the existing merge semantics.

The queue explicitly sorts the drained deltas by `UiNodePath` at the flush boundary. Deterministic
batch serialization, barrier ordering, view provenance, and press/release segment boundaries are
therefore unchanged; ordered-map maintenance is removed from each hot-path insertion.

## Performance Workload

The release workload fills 16,384 realistic UI node paths and performs 4,096 stable coalescing
lookups for the final path.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered pending-path lookups | 4,096 | 0 |
| Hash pending-path lookups | 0 | 4,096 |
| Order projections per insertion | implicit tree maintenance | 0 |
| Explicit order projections per flush | 0 | 1 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR48_UI_DELTA_HASH_COALESCING_BENCH_V1`. Acceptance requires hash lookup P95 to be at least
30% below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the coordinator
run.

## Acceptance

- `optimization_batch_20260826cd_ui_delta_hash_coalescing_preserves_sorted_flush` covers latest
  wins and deterministic path order from intentionally shuffled input.
- `optimization_batch_20260826cd_ui_delta_hash_coalescing_keeps_order_at_flush_only` locks the hash
  owner and explicit flush projection.
- `optimization_batch_20260826cd_ui_delta_hash_coalescing_p95` reports paired release P50/P95
  samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Editor48 still owns frame/generation paging, byte/count budgets, overflow disposition, failed-apply
resync, acknowledgements, and product-scale UI delta latency. This slice only converges the
per-segment node coalescing lookup.
