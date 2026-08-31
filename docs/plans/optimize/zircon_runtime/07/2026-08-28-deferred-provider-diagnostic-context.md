---
title: Runtime07 Deferred Provider Diagnostic Context
category: zircon_runtime
report_id: Runtime07-deferred-provider-diagnostic-context-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Deferred Provider Diagnostic Context

## Scope

This slice removes eager context-string allocation from project plugin selection and feature
provider package-id validation. It preserves selection/feature traversal, validity rules, required
row fatal propagation, diagnostic order, and exact diagnostic text.

## Change

- Pass `format_args!` from selection and feature-provider validation call sites instead of
  allocating context strings with `format!` before validation.
- Accept `fmt::Arguments` in the shared package-id validator and materialize only final diagnostic
  strings inside failing branches.
- Add a Rust regression covering exact selection/provider diagnostic text and a Python source
  contract for deferred formatting at both call sites.

## Deterministic Performance Evidence

The standalone optimized Rust model validates selection and provider package IDs for 65,536 rows
across 17 alternating samples. Every 64th row is invalid, retaining sparse diagnostic output while
exercising the common valid fast path. Complete diagnostic vectors compare byte-for-byte and both
produced checksum `403456`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 71,691 | 6,155 | 91.420% |
| Requested allocation bytes | 6,944,672 | 1,308,576 | 81.160% |
| Validation P50 | 17.9385 ms | 6.3172 ms | 64.780% |
| Validation P95 | 30.9329 ms | 7.6857 ms | 75.150% |

Evidence marker: `RUNTIME07_DEFERRED_PROVIDER_DIAGNOSTIC_CONTEXT_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_deferred_provider_diagnostic_context_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model asserts byte-for-byte equality for the complete sparse-error diagnostic
  vector.
- A Rust regression asserts exact selection and feature-provider diagnostic text.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks passed.
- Managed Rust compilation and focused tests remain pending in a later asynchronous Runtime07 batch;
  this candidate will not be validated alone.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
