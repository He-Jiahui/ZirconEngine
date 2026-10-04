---
title: Runtime75 Collection Single State Resolution
category: zircon_runtime
report_id: Runtime75-collection-single-resolution-2026-09-14
date: 2026-09-14
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Collection Single State Resolution

## Scope

Collection reducer array and map mutations previously resolved the same property from the retained
state once to validate and again to mutate. On a user-driven collection edit, this repeats the
state-map lookup and, for several map operations, a tree lookup. This Runtime75 slice keeps the
resolved mutable container for the full validation-and-mutation decision.

## Implementation

- Made array set, remove, and move validate and mutate through one mutable array resolution.
- Made map add use the BTreeMap Entry API, map set use one mutable lookup, and map remove use one
  removal lookup.
- Retained automatic replacement of non-array/non-map values, bounds clamping for moves,
  validation messages, error variants, reference-source clearing, and ownership of input values.
- Added lower success/error coverage and the ignored
  RUNTIME770_COLLECTION_SINGLE_RESOLUTION_BENCH_V1 Release marker.

## Deterministic work boundary

Each affected mutation reduces retained-state property resolution from two to one; successful map
add/set paths also avoid a separate contains-key pass. This does not alter collection contents,
error behavior, or transaction semantics. It is source/work-shape evidence only, not allocator,
CPU/RSS, or product collection-edit p50/p95/p99 evidence.

## Validation

- The TDD source contract was RED before implementation and is GREEN at 5/5:
  tools/tests/test_runtime_collection_single_resolution_performance_contract.py.
- The combined Runtime/Editor command-palette, keyboard, menu, TreeView, TextInput, selection,
  collection, and adjacent capacity-contract invocation passed 82/82 in 0.829s through
  python -B -m unittest.
- No standalone Cargo process, coordinator status query, or tooling-production change was made.

## Remaining acceptance

The lower Rust regression and ignored Release marker must run in the owner-attributed combined
Runtime/Editor Windows Release lane. Current-source Cargo compilation, allocation behavior, and
product percentile evidence remain required before performance acceptance.
