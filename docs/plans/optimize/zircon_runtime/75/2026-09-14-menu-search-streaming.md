---
title: Runtime75 Menu Search Streaming
category: zircon_runtime
report_id: Runtime75-menu-search-streaming-2026-09-14
date: 2026-09-14
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Menu Search Streaming

## Scope

The menu-search builder recursively returned temporary `Vec<MenuSearchOption>` values through
array and map `flat_map` paths. The root option list is the sole retained owner of top-level search
options, while child lists are owned by their parent options. This Runtime75 slice streams recursive
values into those caller-owned vectors and avoids recursive `top_level_id` buffer cloning.

## Implementation

- `menu_search_option_list` creates one conservative root vector and passes it to the top-level
  recursive builder.
- Top-level and descendant builders append directly to caller-owned output vectors, reserving each
  direct array bound before recursion.
- Descendant recursion borrows the stable top-level ID and clones it only for retained
  `MenuSearchOption` fields. Runtime781 subsequently replaced the fixed child-property reference
  vector with a lazy iterator; child output vectors now stay zero-capacity on empty child maps.
- Top-level indexing, map identity aliases, labels, empty-ID filtering, preorder, child ownership,
  default-focus flags, and search filtering behavior remain unchanged.
- Added a lower tree-order/focus-index regression and ignored
  `RUNTIME764_MENU_SEARCH_STREAM_BENCH_V1` Release marker.

## Deterministic work boundary

A flat `N`-option menu starts with `N` root slots and no longer creates one temporary vector per
recursive array/map layer. Nested child arrays use their existing owning child vector and reserve
local direct bounds when visited. The change does not alter matching/ranking, disabled eligibility,
or live-surface integration and is not allocator, CPU/RSS, or product keyboard p50/p95/p99 evidence.

## Validation

- TDD source contract was RED before implementation and is GREEN at `4/4`:
  `tools/tests/test_runtime_menu_search_streaming_performance_contract.py`.
- The original Runtime/Editor menu, command-palette, keyboard, TreeView, input, and capacity
  contract invocation passed `58/58` in `0.110s`; the Runtime781 follow-up refresh passes
  `120/120` in `0.080s` through `python -B -m unittest`.
- No standalone Cargo process, coordinator status query, or tooling-production change was made.

## Remaining acceptance

The lower Rust regression and ignored Release marker must run in the owner-attributed combined
Runtime/Editor Windows Release lane. Current-source Cargo compilation, allocation behavior, and
product percentile evidence remain required before performance acceptance.
