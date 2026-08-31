---
title: Runtime07 Borrowed Reflection Owner Validation
category: zircon_runtime
report_id: Runtime07-borrowed-reflection-owner-validation-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Borrowed Reflection Owner Validation

## Scope

This slice removes an eager type-path clone from every successful VM reflection package-owner
validation. It preserves registration-owner and type-path-owner checks, missing-owner diagnostics,
validation order, and the complete owned `PackageOwnerMismatch` error payload.

## Change

- Keep each reflected type path borrowed while both owner checks succeed.
- Clone the type path only inside the registration-owner or type-path-owner error constructor that
  needs to return an owned diagnostic.
- Preserve the existing expected-owner and declared-owner string construction on error paths.
- Add a Python performance contract for the allocation boundary and retain the existing Rust
  foreign-namespace regression as the behavior guard.

## Deterministic Performance Evidence

The standalone optimized Rust model validates 65,536 registrations owned by one package across 31
alternating samples. It also compares complete error payloads for registration-owner and type-path-
owner mismatches. Both implementations produced checksum `2686976`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 65,536 | 0 | 100.000% |
| Requested allocation bytes | 2,686,976 | 0 | 100.000% |
| Run 1 validation P50 | 13.9987 ms | 3.5719 ms | 74.484% |
| Run 1 validation P95 | 53.8650 ms | 48.1863 ms | 10.542% |
| Run 2 validation P50 | 16.6457 ms | 3.9229 ms | 76.433% |
| Run 2 validation P95 | 19.5390 ms | 6.5316 ms | 66.571% |

Evidence marker: `RUNTIME07_BORROWED_REFLECTION_OWNER_VALIDATION_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_borrowed_reflection_owner_validation_performance_contract.py`:
  3 passed after the pre-change contract failed 2 of 3 checks.
- The standalone Rust model asserts success checksums and both owned mismatch payloads before
  recording metrics; two runs retained zero allocations and positive P50/P95 results.
- Existing Rust regression
  `trusted_package_owner_rejects_self_consistent_foreign_namespaces` remains the error behavior
  guard.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required before
  snapshot publication.
- Managed Rust compilation and the focused reflection regression remain pending in a later
  asynchronous Runtime07 batch with another completed optimization.

Managed batch request: `runtime07-borrowed-gameplay-seven-task-batch-20260830-v1`.

Validation attempt: ticket `a9dc9a55e9044c239cc7dfda8bbc64b6` failed before Cargo at
coordinator artifact governance for `D:\ZirconBuilds\mvp-test-fixtures-36724`; the 22 local contract
checks remain green while integrated acceptance and success publication remain pending.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
