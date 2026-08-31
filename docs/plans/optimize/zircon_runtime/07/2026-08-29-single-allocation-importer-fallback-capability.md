---
title: Runtime07 Single-allocation Importer Fallback Capability
category: zircon_runtime
report_id: Runtime07-single-allocation-importer-fallback-capability-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Single-allocation Importer Fallback Capability

## Scope

This slice removes the intermediate owned slug from fallback importer capability construction.
Unknown importer ids previously allocated once for `_` to `.` replacement and again for the
`runtime.asset.importer.` prefix projection during built-in catalog classification.

## Change

- Keep the explicit capabilities for known first-party importers unchanged.
- Borrow the fallback slug after removing the optional `_importer` suffix.
- Allocate one output string for the exact prefix-plus-slug byte length.
- Translate underscores to dots while writing the final output, avoiding an intermediate
  `String` and formatting machinery.
- Preserve ids with and without the importer suffix, including non-underscore characters.
- Add a Rust regression for a suffixed fallback, an unsuffixed fallback, and a known importer.
- Add a Python source performance contract for the one-output construction path.

## Deterministic Performance Evidence

The standalone optimized Rust model builds 262,144 long fallback importer capabilities per
sample across 31 alternating samples. Equivalence is also checked for suffixed and unsuffixed
fallback ids. Both implementations produced checksum `7956002070714928598`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per 131,072 builds | 393,216 | 131,072 | 66.667% |
| Requested allocation bytes | 23,199,744 | 8,126,464 | 64.972% |
| Run 1 build P50 | 82.7548 ms | 50.2548 ms | 39.273% |
| Run 1 build P95 | 145.9739 ms | 142.1791 ms | 2.600% |
| Run 2 build P50 | 91.1591 ms | 53.6237 ms | 41.176% |
| Run 2 build P95 | 145.3018 ms | 100.8046 ms | 30.624% |

Evidence marker: `RUNTIME07_SINGLE_ALLOCATION_IMPORTER_FALLBACK_CAPABILITY_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_single_allocation_importer_fallback_capability_performance_contract.py`:
  4 passed after all 4 pre-change checks failed.
- `python -m py_compile` passed for the source contract.
- The standalone Rust model retained identical fallback capability text; two runs kept the same
  allocation profile and checksum, with positive P50/P95 results in both runs.
- The Rust regression locks suffixed, unsuffixed, and known-importer behavior.
- Exact-file Rust formatting, model formatting, and scoped diff checks are required before
  snapshot publication.
- Managed Runtime compilation and tests remain pending in the next asynchronous Runtime07 batch.

Managed batch request: `runtime07-plugin-five-task-batch-20260830-v1`.

Validation attempt: ticket `27e27a159794475b9bd8636cf2859288` failed before Cargo at
coordinator artifact governance for `D:\ZirconBuilds\mvp-test-fixtures-36724`; integrated acceptance
and success publication remain pending.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
