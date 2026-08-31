---
title: Editor01 Invalidation Scope Hash Index
category: zircon_editor
report_id: Editor01-invalidation-scope-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 Invalidation Scope Hash Index

## Scope

This slice replaces the retained host's pending invalidation-scope owner with `HashMap`. Repeated
view, shell-content, and global invalidation requests now coalesce and resolve through expected
constant-time lookup.

`HostInvalidationScope` hashes exactly the fields used by equality. Mask union and consume behavior
remain order-independent. The existing presentation-only view snapshot remains deterministically
sorted, while the single shell-content fast path still accepts exactly one matching scope.

## Performance Workload

The release workload fills 4,096 long shared-prefix view scopes and performs 4,096 stable hits for
the final production `HostInvalidationScope` key.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered invalidation-scope lookups | 4,096 | 0 |
| Hash invalidation-scope lookups | 0 | 4,096 |
| Presentation snapshot ordering changes | 0 | 0 |
| Allocations on scope hits | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR01_INVALIDATION_SCOPE_HASH_INDEX_BENCH_V1`. Acceptance requires hash lookup P95 to be at
least 30% below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826by_invalidation_scope_hash_index_preserves_sorted_view_snapshot`
  covers coalescing, distinct-scope count, and deterministic view snapshot order.
- `optimization_batch_20260826by_invalidation_scope_hash_index_keeps_explicit_snapshot_order`
  locks the hash owner and explicit snapshot-sort contract.
- `optimization_batch_20260826by_invalidation_scope_hash_index_p95` reports paired release P50/P95
  samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Editor01 still owns retained invalidation, layout and paint scaling, shared visual materialization,
accessibility, and product-scale interaction evidence. This slice only converges pending
invalidation-scope lookup and coalescing.
