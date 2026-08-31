---
title: Runtime07 Preallocated Catalog Selection Completion
category: zircon_runtime
report_id: Runtime07-preallocated-catalog-selection-completion-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Preallocated Catalog Selection Completion

## Scope

This slice removes geometric container growth when a partial or empty project plugin manifest is
completed from catalog registrations. It preserves owned selection IDs, registration order, first
selection semantics, duplicate suppression, cloned selection payloads, and the disabled default for
newly synthesized selections.

## Change

- Size the selected-package ID set from the complete upper bound of existing selections plus catalog
  registrations before copying owned IDs.
- Reserve the same registration upper bound in the completed selection vector before appending
  missing selections.
- Keep the existing registration-order traversal and owned ID clones required by the mutable target
  vector.
- Add a Rust source regression plus a Python performance contract covering both preallocations and
  the unchanged order/default behavior shape.

## Deterministic Performance Evidence

The standalone optimized Rust model completes an empty project manifest from 65,536 registrations
across 31 alternating samples. Each modeled selection retains two owned `String` fields, so the
measurement includes the unavoidable selection and ID cloning cost. Both paths produced identical
ordered selections and checksum `2818048`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 196,639 | 196,610 | 0.015% |
| Requested allocation bytes | 18,022,332 | 11,075,600 | 38.545% |
| Completion P50 | 75.2524 ms | 52.2148 ms | 30.614% |
| Completion P95 | 187.8624 ms | 106.4525 ms | 43.335% |

Evidence marker: `RUNTIME07_PREALLOCATED_CATALOG_SELECTION_COMPLETION_MODEL_V1`.

Allocation-call totals remain dominated by the two required owned strings per completed selection;
the removed 29 calls are container growth events. Their cumulative requested bytes and latency are
large enough to remain material at catalog scale.

## Validation

- `python tools/tests/test_runtime07_preallocated_catalog_selection_completion_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model asserts exact ordered selection equality before recording metrics.
- Existing catalog tests retain duplicate-selection, first-feature-match, completion report, and
  extension-order behavior coverage.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks passed.
- Managed Rust compilation and focused tests remain pending in a later asynchronous Runtime07 batch;
  this candidate will be paired with another completed optimization.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
