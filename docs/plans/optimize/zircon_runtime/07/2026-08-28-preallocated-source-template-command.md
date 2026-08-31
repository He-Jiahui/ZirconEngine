---
title: Runtime07 Preallocated Source Template Command
category: zircon_runtime
report_id: Runtime07-preallocated-source-template-command-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Preallocated Source Template Command

## Scope

This slice removes dynamic path normalization and avoidable command-vector growth from source
template build-plan generation. It preserves manifest and target paths, Debug/Release profiles,
command argument order, and owned plan fields.

## Change

- Replace the fixed `Path::join -> display -> String::replace` path pipeline with canonical static
  manifest and forward-slash target paths.
- Build the command in a dedicated helper with exact capacity for six Debug arguments or seven
  Release arguments.
- Materialize owned plan fields directly from the same canonical constants.
- Add a Rust regression for the complete Release command and a Python source contract for the fixed
  path and capacity invariants.

## Deterministic Performance Evidence

The standalone optimized Rust model constructs 32,768 alternating Debug/Release plans for 17
alternating samples. Both implementations first compare the complete plan structures. Both produced
checksum `4128768`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 557,056 | 344,064 | 38.235% |
| Requested allocation bytes | 18,038,784 | 9,224,192 | 48.865% |
| Plan generation P50 | 63.8820 ms | 25.0804 ms | 60.739% |
| Plan generation P95 | 136.4879 ms | 59.2357 ms | 56.600% |

Evidence marker: `RUNTIME07_PREALLOCATED_SOURCE_TEMPLATE_COMMAND_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_preallocated_source_template_command_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model asserts complete Debug and Release plan equality.
- A Rust regression asserts the complete Release command contract.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks passed.
- Managed Rust compilation and focused tests remain pending in a later asynchronous Runtime07 batch;
  this candidate will not be validated alone.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
