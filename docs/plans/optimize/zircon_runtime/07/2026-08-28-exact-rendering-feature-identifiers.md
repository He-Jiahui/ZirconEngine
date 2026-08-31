---
title: Runtime07 Exact Rendering Feature Identifiers
category: zircon_runtime
report_id: Runtime07-exact-rendering-feature-identifiers-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Exact Rendering Feature Identifiers

## Scope

This slice removes formatter growth from the seven owned identifiers built for every builtin
rendering feature row. It preserves feature IDs, runtime/editor capabilities, runtime/editor crate
names, module names, declaration order, target modes, dependencies, and default-enable policy.

## Change

- Add one small identifier join helper that sums borrowed part lengths, allocates the final `String`
  once, and appends every part in order.
- Route all seven rendering feature identifier constructions through that exact-capacity path.
- Add a Rust exact-output regression and a Python structure contract preventing formatter-based
  identifier construction from returning to this manifest builder.

## Deterministic Performance Evidence

The standalone optimized Rust model cycles all 15 production rendering feature suffixes across
65,536 rows per sample and constructs the same seven identifiers as the manifest builder. It
alternates legacy and optimized order across 31 samples, counts allocator calls and requested
bytes inside identifier construction, and asserts exact seven-string equality for every production
suffix. Both paths produced checksum `2163129653512`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 703,420 | 524,288 | 25.466% |
| Requested allocation bytes | 28,665,451 | 17,821,416 | 37.830% |
| Identifier construction P50 | 76.3945 ms | 41.9699 ms | 45.062% |
| Identifier construction P95 | 93.4072 ms | 52.8679 ms | 43.401% |

Evidence marker: `RUNTIME07_EXACT_RENDERING_FEATURE_IDENTIFIERS_MODEL_V1`.

A second complete run remained favorable: P50 improved 38.095% and P95 improved 68.190%.

## Validation

- `python tools/tests/test_runtime07_exact_rendering_feature_identifiers_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model compiled with Rust 1.94.1, asserted exact identifier arrays for all
  production suffixes, and passed two complete 31-sample runs.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot freeze.
- Managed Rust compilation and focused tests remain pending in the next asynchronous Runtime07
  validation batch.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
