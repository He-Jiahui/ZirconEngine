---
title: Runtime07 Preallocated Builtin Host Module Handles
category: zircon_runtime
report_id: Runtime07-preallocated-builtin-host-module-handles-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Preallocated Builtin Host Module Handles

## Scope

This slice removes the second allocation from the builtin script host-module handle collector
during a complete installation. The five fixed runtime modules plus the gameplay module establish
an exact upper bound of six handles while preserving conditional registration and return order.

## Change

- Define the complete-install handle capacity as six.
- Allocate the result vector at that exact capacity instead of growing through the default 4-to-8
  capacity transition.
- Preserve every module existence check, registration call, error boundary, and output ordering.
- Add a Rust regression binding the five fixed module identities plus gameplay to the capacity.
- Add a Python source performance contract for the exact preallocation and six append sites.

## Deterministic Performance Evidence

The standalone optimized Rust model builds 262,144 six-handle collectors per sample across 31
alternating samples. Both implementations preserve exact handle order and produced checksum
`828444533728879397`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per 65,536 collectors | 131,072 | 65,536 | 50.000% |
| Requested allocation bytes | 6,291,456 | 3,145,728 | 50.000% |
| Run 1 collection P50 | 63.6552 ms | 23.5564 ms | 62.994% |
| Run 1 collection P95 | 154.0825 ms | 51.3423 ms | 66.679% |
| Run 2 collection P50 | 72.0519 ms | 26.4623 ms | 63.273% |
| Run 2 collection P95 | 205.0899 ms | 149.4651 ms | 27.122% |

Evidence marker: `RUNTIME07_PREALLOCATED_BUILTIN_HOST_MODULE_HANDLES_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_preallocated_builtin_host_module_handles_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model asserts exact handle order and contents before measurement; two runs
  retained identical allocation profiles, checksums, and positive P50/P95 results.
- The Rust regression ties all fixed builtin module identities plus gameplay to the capacity.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot publication.
- Managed Rust compilation and focused builtin host-module tests remain pending in the next
  asynchronous Runtime07 validation batch.

Managed batch request: `runtime07-native-vm-six-task-batch-20260830-v1`.

Validation attempt: ticket `167f127a7c8d48b3a68554a5c4f1d0f7` failed during coordinator
materialization with `unmanaged_artifacts_detected` for
`D:\ZirconBuilds\mvp-test-fixtures-36724`; Cargo did not start, so integrated Rust and performance
acceptance remain pending.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
