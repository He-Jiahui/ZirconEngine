---
title: Runtime75 Menu Search Query Borrow
category: zircon_runtime
report_id: Runtime75-menu-search-query-borrow-2026-09-15
date: 2026-09-15
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Menu Search Query Borrow

## Scope

Menu search-filter synchronization cloned the retained `search_query` setting before trimming and
lowercasing it. The clone was unnecessary for lookup and also occurred for whitespace-only values
that are discarded by the final query filter.

## Implementation

- Added a borrowed nonempty setting lookup that preserves state-over-descriptor-default precedence
  while treating empty stored values as absent, matching the prior setting helper.
- Trims the borrowed query before allocating its lowercase result, so empty/whitespace-only input
  returns `None` without a normalized `String` allocation.
- Preserves query trimming, Unicode lowercase expansion, empty-query behavior, and filter/focus
  semantics.
- Added lower state/default/whitespace regressions and ignored marker
  `RUNTIME778_MENU_SEARCH_QUERY_BORROW_BENCH_V1`.

## Deterministic work model

For a nonempty query setting, the path removes the temporary clone of the authored/state string and
allocates only the required normalized query. Empty or whitespace-only values are rejected before
normalization. This is allocation-shape evidence only, not a claim about allocator, CPU/RSS, or
product menu-search latency percentiles.

## Validation

- TDD source contract:
  `tools/tests/test_runtime_menu_search_query_borrow_performance_contract.py` was RED before
  implementation and is GREEN at `2/2`.
- The combined Runtime/Editor source-contract batch passes `120/120` in `0.080s`.
- `rustfmt --edition 2021 --check` passes for the production menu reducer and lower regression.
- Managed Cargo, Release allocation, and product p50/p95/p99 evidence remain pending in the
  owner-attributed Runtime/Editor batch.

## Remaining acceptance

The owner-attributed Windows Runtime/Editor lane must compile the current source, execute the
lower regression and ignored marker, and report the plan-specific allocation and latency gates.
