---
title: Runtime07 Preallocated Registration Interface Sets
category: zircon_runtime
report_id: Runtime07-preallocated-registration-interface-sets-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Preallocated Registration Interface Sets

## Scope

This slice removes repeated growth from the exported and imported interface sets built during
runtime plugin registration validation. It preserves runtime-module filtering, borrowed interface
identifiers, missing-owner behavior, duplicate elimination, diagnostic order, and validation text.

## Change

- Retain each registry iterator long enough to read its lower-bound size hint before traversal.
- Preallocate the exported and imported `HashSet` values with those bounds and insert borrowed
  interface identifiers in the existing filter order.
- Keep non-runtime and unresolved-owner rows out of the validation sets without cloning identifiers.
- Add a Rust source regression plus a Python performance contract covering both preallocated sets
  and the existing registration-validation behavior tests.

## Deterministic Performance Evidence

The standalone optimized Rust model builds exported and imported sets from 65,536 rows, with every
eighth row excluded as non-runtime, across 31 alternating samples. Both paths produced identical
sets and checksum `2981888`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 30 | 2 | 93.333% |
| Requested allocation bytes | 4,456,792 | 4,456,480 | 0.007% |
| Set-build P50 | 43.2611 ms | 27.2470 ms | 37.017% |
| Set-build P95 | 70.0909 ms | 42.1433 ms | 39.873% |

Evidence marker: `RUNTIME07_PREALLOCATED_REGISTRATION_INTERFACE_SETS_MODEL_V1`.

The final requested bytes are intentionally almost unchanged because both implementations retain
the same large sets. The gain is from replacing 28 intermediate growth allocations with the two
final set allocations.

## Validation

- `python tools/tests/test_runtime07_preallocated_registration_interface_sets_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model asserts exact equality for both interface sets before recording metrics.
- Existing Rust behavior tests for undeclared and unregistered interface imports remain in place.
- Exact-file Rust formatting and scoped diff checks passed.
- Managed Rust compilation and focused tests remain pending in a later asynchronous Runtime07 batch;
  this candidate will be validated with another completed optimization.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
