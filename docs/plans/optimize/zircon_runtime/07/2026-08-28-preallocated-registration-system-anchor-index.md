---
title: Runtime07 Preallocated Registration System Anchor Index
category: zircon_runtime
report_id: Runtime07-preallocated-registration-system-anchor-index-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Preallocated Registration System Anchor Index

## Scope

This slice removes geometric growth from the temporary registered-system anchor index built during
runtime plugin registration validation. It preserves both registration kinds, unresolved-owner
filtering, borrowed module/system identifiers, duplicate elimination, diagnostic order, and exact
diagnostic text.

## Change

- Retain the plugin-system and runtime-system iterators long enough to read their upper size bounds.
- Allocate the borrowed `HashSet<(&str, &str)>` once from the combined bound before traversing rows.
- Insert both registration kinds in separate single-pass loops because their descriptor types are
  distinct, while preserving the existing module-owner resolution behavior.
- Add a Rust source regression plus a Python performance contract that prevents a return to the
  zero-lower-bound `filter_map().chain().collect()` path.

## Deterministic Performance Evidence

The standalone optimized Rust model builds the anchor index from 65,536 rows split across both
registration kinds, with every sixty-fourth row lacking a resolved owner, across 31 alternating
samples. Both paths produced identical borrowed tuple sets and checksum `2456320`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 16 | 1 | 93.750% |
| Requested allocation bytes | 8,650,876 | 4,325,392 | 50.001% |
| Index-build P50 | 50.0589 ms | 19.6043 ms | 60.838% |
| Index-build P95 | 65.6204 ms | 28.2440 ms | 56.959% |

Evidence marker: `RUNTIME07_PREALLOCATED_REGISTRATION_SYSTEM_ANCHOR_INDEX_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_preallocated_registration_system_anchor_index_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model asserts exact set equality before allocation and latency sampling.
- The Rust source regression retains both capacity hints, borrowed insertions, and the absence of
  the unbounded collection shape.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks passed.
- Managed Rust compilation and focused tests remain pending in an asynchronous Runtime07 batch with
  the preallocated registration interface-set candidate.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
