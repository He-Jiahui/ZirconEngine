---
title: Runtime74 Component Contract Hash Index
category: zircon_runtime
report_id: Runtime74-component-contract-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime74 Component Contract Hash Index

## Scope

This slice replaces ordered component-tree membership indexes with borrowed hash indexes during UI
component contract validation. Tree traversal, public-part iteration, selector diagnostics, error
priority, and document publication order remain unchanged.

## Change

- Store borrowed node IDs and control IDs in `HashSet<&str>`.
- Store the borrowed node-to-control relation in `HashMap<&str, &str>`.
- Keep first/last insertion behavior identical: duplicate keys still retain membership and the
  node-to-control map still holds the last visited mapping.
- Store private and public target membership in borrowed hash sets; neither set publishes order.

## Deterministic Performance Evidence

| Representative component tree | Before | After |
|---|---:|---:|
| 4,096 direct children plus root, three indexes | `O(N log N)` ordered inserts | average `O(N)` hash inserts |
| 12,288 hit/miss probes across three indexes | 36,864 `O(log N)` lookups | 36,864 average `O(1)` lookups |
| Borrowed ID allocations | 0 | 0 |
| Contract diagnostic traversal | document/source order | unchanged |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME74_COMPONENT_CONTRACT_HASH_INDEX_BENCH_V1`. Acceptance requires both index-build P95 and
lookup P95 to be at most 60% of their ordered-index baselines. Exact Windows timings remain pending
the coordinator run.

## Acceptance

- `optimization_batch_20260826ak_component_tree_hash_index_preserves_targets` covers node,
  control, node-to-control, control-target, and missing-node behavior.
- `optimization_batch_20260826ak_component_contract_uses_borrowed_hash_indexes` requires all four
  borrowed hash-index boundaries and rejects ordered node membership.
- `optimization_batch_20260826ak_component_contract_hash_index_p95` reports four P95 values and
  enforces both 60% thresholds.

## Remaining Parent-plan Work

Runtime74 still owns complete template compiler/runtime authoring, compatibility, hot reload,
binding, event, and product-scale performance gates. This slice only converges component-contract
validation membership.
