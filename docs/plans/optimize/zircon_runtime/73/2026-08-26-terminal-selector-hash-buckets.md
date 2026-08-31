---
title: Runtime73 Terminal Selector Hash Buckets
category: zircon_runtime
report_id: Runtime73-terminal-selector-hash-buckets-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime73 Terminal Selector Hash Buckets

## Scope

This slice replaces the four private terminal-selector bucket maps with `HashMap`. Style matching
performs ID, class, type, and state point lookups for every resolved node, so these indexes have no
ordered traversal requirement.

Candidate rule indices are still sorted and deduplicated after all matching buckets are combined.
The unchanged full selector matcher remains the final oracle, preserving cascade/source order,
ancestor matching, host handling, and fail-closed token behavior.

## Performance Workload

The release workload fills 16,384 realistic selector keys and performs 4,096 stable lookups for
the final key.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered selector-bucket lookups | 4,096 | 0 |
| Hash selector-bucket lookups | 0 | 4,096 |
| Candidate sort/dedup projections | unchanged | unchanged |
| Full selector matcher semantics | unchanged | unchanged |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME73_SELECTOR_HASH_BUCKETS_BENCH_V1`. Acceptance requires hash lookup P95 to be at least 30%
below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826cd_selector_hash_buckets_preserve_candidate_order` covers ID, class,
  type, state, nonmatching rules, and original candidate order.
- `optimization_batch_20260826cd_selector_hash_buckets_keep_explicit_order_projection` locks the
  hash indexes plus the explicit sort/dedup boundary.
- `optimization_batch_20260826cd_selector_hash_buckets_p95` reports paired release P50/P95 samples
  and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Runtime73 still owns compiled ancestor predicates, state dependency indexing, typed selector
bytecode, computed-style sharing, transition policy, and product-scale qualification. This slice
only converges the private terminal-selector lookup buckets.
