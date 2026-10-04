---
title: Runtime75 Tree ID Collection Capacity
category: zircon_runtime
report_id: Runtime75-tree-id-collection-capacity-2026-09-14
date: 2026-09-14
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Tree ID Collection Capacity

## Scope

TreeView reduces nested `UiValue` declarations into ordered node, selected, and disabled ID
collections. The recursive collectors already know the cardinality of each direct array or flags
container just before traversing it, but formerly allowed their `Vec` and `HashSet` outputs to
grow geometrically from zero. This Runtime75 slice reserves only that direct bound; it does not
pre-scan or flatten a nested declaration tree.

## Implementation

- `collect_tree_node_ids` reserves direct array capacity for both its ordered output and
  deduplication set before recursion.
- `collect_borrowed_string_ids` and `collect_owned_string_ids` do the same for direct arrays and
  flags lists.
- `collect_disabled_option_ids` reserves direct array and flags bounds before its existing insert
  or extend paths.
- Existing recursion, first-occurrence ordering, duplicate suppression, empty-value handling, and
  the TreeView-specific distinction that node collection ignores flags remain unchanged.
- Added a lower semantic/capacity regression and ignored
  `RUNTIME757_TREE_ID_COLLECTION_CAPACITY_BENCH_V1` Release marker.

## Deterministic work boundary

For a direct container with `N` values, the collector requests room for `N` additional output and,
where applicable, deduplication entries before visiting that container. That removes modeled
growth events attributable to a fully admitted direct container without adding a second recursive
walk. Map-owned child declarations and arbitrary nested expansion intentionally retain their
existing conservative behavior. This is an allocation-shape improvement, not a claim about
allocator counts, CPU/RSS, or product input-to-present p50/p95/p99.

## Validation

- The TDD source contract was introduced before the implementation and is GREEN at `4/4`:
  `tools/tests/test_runtime_tree_id_collection_capacity_performance_contract.py`. Its parser was
  corrected to recognize the existing lifetime-generic collector signatures.
- One adjacent Runtime/Editor command-palette, input, TreeView, and capacity-contract invocation
  passed `42/42` in `0.314s` through `python -B -m unittest`.
- `rustfmt --edition 2021 --check` passes for both changed Runtime reducer files.
- No standalone Cargo process, coordinator status query, or tooling-production change was made.

## Remaining acceptance

The lower Rust regression and ignored Release marker must run in the owner-attributed combined
Runtime/Editor Windows Release lane. Current-source Cargo compilation, allocation behavior, and
product percentile evidence remain required before this slice can be considered performance
accepted.
