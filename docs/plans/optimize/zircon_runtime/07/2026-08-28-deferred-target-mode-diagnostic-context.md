---
title: Runtime07 Deferred Target Mode Diagnostic Context
category: zircon_runtime
report_id: Runtime07-deferred-target-mode-diagnostic-context-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Deferred Target Mode Diagnostic Context

## Scope

This slice removes eager context-string allocation from project plugin and feature target-mode
validation. It preserves target filtering, duplicate diagnostic multiplicity and order, required-row
fatal propagation, exact diagnostic text, and the existing prefix-scan/bitset optimization.

## Change

- Pass `format_args!` from selection and feature target-mode call sites instead of allocating a
  context `String` for every valid row.
- Accept `fmt::Arguments` in the private target-mode validator, materializing context only when a
  duplicate target mode emits a diagnostic.
- Update the existing Runtime231 regression call to the borrowed formatting contract and add an
  exact diagnostic regression plus a Python source contract.

## Deterministic Performance Evidence

The standalone optimized Rust model validates both selection and feature target modes for 65,536
rows across 17 alternating samples. Every 64th row contains a duplicate target mode, retaining sparse
diagnostic output while exercising the common valid fast path. Complete diagnostic vectors compare
byte-for-byte and both produced checksum `183968`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 135,178 | 4,106 | 96.963% |
| Requested allocation bytes | 8,843,168 | 454,560 | 94.860% |
| Validation P50 | 24.7446 ms | 1.7635 ms | 92.873% |
| Validation P95 | 36.3857 ms | 4.2509 ms | 88.317% |

Evidence marker: `RUNTIME07_DEFERRED_TARGET_MODE_DIAGNOSTIC_CONTEXT_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_deferred_target_mode_diagnostic_context_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model asserts byte-for-byte equality for the complete sparse-error diagnostic
  vector.
- A Rust regression asserts the exact feature target-mode duplicate diagnostic.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks passed.
- Managed Rust compilation and focused tests remain pending in a later asynchronous Runtime07 batch;
  this candidate will not be validated alone.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
