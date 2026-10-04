---
title: Runtime75 Menu Search Filter Accumulator
category: zircon_runtime
report_id: Runtime75-menu-search-filter-accumulator-2026-09-15
date: 2026-09-15
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Menu Search Filter Accumulator

## Scope

Recursive menu search filtering allocated a fresh `MenuSearchBuild` for every option, then
extended the parent's vectors with the child's temporary vectors. Deep or broad option trees paid
for repeated temporary vector allocation and movement.

## Implementation

- Recursion now writes directly into the caller-owned `MenuSearchBuild` vectors.
- Checkpoint lengths roll back an option's provisional parent ID and focus output when neither the
  option nor any descendant matches, preserving the prior filtering contract.
- Matching parent IDs and focus candidates are emitted before descendants, retaining preorder and
  focus ordering; unmatched leaves exit before cloning their IDs.
- Added lower preorder/focus-order regression and ignored marker
  `RUNTIME779_MENU_SEARCH_FILTER_ACCUMULATOR_BENCH_V1`.

## Deterministic work model

The recursive path no longer creates one child result container per visited option. It retains one
root accumulator and uses length checkpoints for rollback. This is allocation-shape evidence only,
not a claim about allocator, CPU/RSS, or product menu-search latency percentiles.

## Validation

- TDD source contract:
  `tools/tests/test_runtime_menu_search_filter_accumulator_performance_contract.py` was RED
  before implementation and is GREEN at `3/3`.
- The combined Runtime/Editor source-contract batch passes `120/120` in `0.080s`.
- `rustfmt --edition 2021 --check` passes for the production menu reducer and lower regression.
- Managed Cargo, Release allocation, and product p50/p95/p99 evidence remain pending in the
  owner-attributed Runtime/Editor batch.

## Remaining acceptance

The owner-attributed Windows Runtime/Editor lane must compile the current source, execute the
lower regression and ignored marker, and report the plan-specific allocation and latency gates.
