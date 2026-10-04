---
title: Runtime75 Menu Search Projection Capacity
category: zircon_runtime
report_id: Runtime75-menu-search-projection-capacity-2026-09-14
date: 2026-09-14
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Menu Search Projection Capacity

## Scope

Menu keyboard search repeatedly converts a retained option tree into filtered-ID membership and a
flattened search-ID projection. The recursive collector knew the direct array/flags cardinality
at each entry and the flattened projection knew its root/child counts, but its outputs previously
started at zero capacity. This Runtime75 slice reserves only local known bounds and does not
pre-scan the menu tree.

## Implementation

- `collect_filtered_option_id_refs` reserves each direct array and flags bound before adding to its
  `HashSet`.
- `all_search_option_ids` starts from the top-level option bound; its recursive appender reserves
  each option's direct child count before descent.
- Search filtering, deduplication, preorder traversal, focus-candidate selection, and empty-query
  behavior remain unchanged.
- Added a lower nested-ID/capacity regression and ignored
  `RUNTIME763_MENU_SEARCH_PROJECTION_CAPACITY_BENCH_V1` Release marker.

## Deterministic work boundary

For direct containers with `N` candidates, the output has room for up to `N` local additions before
that traversal occurs. A nested tree is still visited once in its existing preorder; no global
counting pass, query algorithm, or focus policy changes. This is allocation-shape evidence only,
not allocator, CPU/RSS, or product keyboard p50/p95/p99 evidence.

## Validation

- TDD source contract was RED before implementation and is GREEN at `4/4`:
  `tools/tests/test_runtime_menu_search_capacity_performance_contract.py`.
- The combined Runtime/Editor menu, command-palette, keyboard, TreeView, input, and capacity
  contract invocation passed `58/58` in `0.110s` through `python -B -m unittest`.
- No standalone Cargo process, coordinator status query, or tooling-production change was made.

## Remaining acceptance

The lower Rust regression and ignored Release marker must run in the owner-attributed combined
Runtime/Editor Windows Release lane. Current-source Cargo compilation, allocation behavior, and
product percentile evidence remain required before performance acceptance.
