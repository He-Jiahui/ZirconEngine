---
title: Runtime07 Preallocated Unresolved Feature Collection
category: zircon_runtime
report_id: Runtime07-preallocated-unresolved-feature-collection-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Preallocated Unresolved Feature Collection

## Scope

This slice removes geometric result-vector growth when unresolved runtime plugin features are
handed to the blocking pass. It preserves feature-index order, available-feature wakeups,
immediately blocked handling, empty-slot omission, dependency diagnostics, and resolution stats.

## Change

- Track the exact number of occupied unresolved states while waiting features enter the indexed
  state vector and ready features leave it.
- Allocate the final unresolved vector once from that count and extend it from the existing
  `Option` slots instead of collecting a flattened iterator whose lower size hint is zero.
- Centralize the capacity invariant in a small generic helper with a debug assertion that the
  tracked count matches the materialized count.
- Add a Rust behavior regression proving empty-slot omission and original index order, plus a
  Python performance structure contract for the counter and preallocation path.

## Deterministic Performance Evidence

The standalone optimized Rust model processes 65,536 indexed states, including initially empty
states and ready states removed before the blocking pass. It leaves 44,938 unresolved records,
models each record with a 64-byte movable payload, includes the optimized counter maintenance in
the timed region, alternates legacy and optimized order across 31 samples, and asserts exact vector
equality for every pair. Both paths produced checksum `5776990817672645335`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 16 | 2 | 87.500% |
| Requested allocation bytes | 13,106,944 | 7,594,624 | 42.056% |
| Resolution-state P50 | 3.6858 ms | 2.6822 ms | 27.229% |
| Resolution-state P95 | 10.1740 ms | 3.6929 ms | 63.703% |

Evidence marker: `RUNTIME07_PREALLOCATED_UNRESOLVED_FEATURE_COLLECTION_MODEL_V1`.

A second full run remained favorable after including counter updates: P50 improved 29.455% and P95
improved 42.570%.

## Validation

- `python tools/tests/test_runtime07_preallocated_unresolved_feature_collection_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model compiled with Rust 1.94.1, asserts exact ordered state equality, and
  passed two complete 31-sample runs.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot freeze.
- Managed Rust compilation and focused tests remain pending in one asynchronous Runtime07 batch
  paired with the preallocated host capability snapshot.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
