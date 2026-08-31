---
title: Runtime07 Preallocated Profile Feature Indexes
category: zircon_runtime
report_id: Runtime07-preallocated-profile-feature-indexes-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Preallocated Profile Feature Indexes

## Scope

This slice removes repeated hash-table growth while building qualified and short feature-id indexes
for each export-profile owner. It preserves owner ordering independence, qualified/short
classification, duplicate elimination, and allocation-free lookup behavior.

## Change

- Centralize per-owner feature indexing in `SelectedProfileFeatureIds::from_feature_ids`.
- Count qualified identifiers first and derive the short count from the source length.
- Allocate each hash set at its exact category count before cloning and inserting identifiers.
- Keep the existing `contains('.')` classification and profile lookup logic unchanged.
- Add a Rust regression for mixed qualified/short identifiers and a Python source contract for exact
  category capacities and helper routing.

## Deterministic Performance Evidence

The standalone optimized Rust model builds the complete feature index for 1,024 owners with 64
mixed identifiers each, or 65,536 identifiers total, across 17 alternating samples. Both paths first
compare every owner and both hash sets. Both produced checksum `1125376`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 76,801 | 68,609 | 10.667% |
| Requested allocation bytes | 7,885,840 | 4,682,768 | 40.618% |
| Index construction P50 | 26.6400 ms | 21.6604 ms | 18.692% |
| Index construction P95 | 29.9660 ms | 28.8882 ms | 3.597% |

Evidence marker: `RUNTIME07_PREALLOCATED_PROFILE_FEATURE_INDEXES_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_preallocated_profile_feature_indexes_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model asserts full outer-map and qualified/short-set equality.
- A Rust regression asserts mixed qualified and short identifier membership.
- Existing linear build/lookup, allocation-free matching, and short/qualified lookup tests remain in
  the module for managed execution.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks passed.
- Managed Rust compilation and focused tests remain pending in an asynchronous Runtime07 batch with
  the single-pass linked-runtime projection candidate.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
