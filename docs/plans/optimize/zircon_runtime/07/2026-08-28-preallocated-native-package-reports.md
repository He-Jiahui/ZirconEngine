---
title: Runtime07 Preallocated Native Package Reports
category: zircon_runtime
report_id: Runtime07-preallocated-native-package-reports-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Preallocated Native Package Reports

## Scope

This slice removes avoidable container growth from non-ZIP native package materialization and
preview. It does not change package discovery, copy order, report paths, diagnostics, or
filesystem behavior.

## Change

- Preallocate materialize and preview package-directory deduplication sets from the selected native
  package count.
- Reserve the copied-package report vector immediately before native package processing and only
  when the native package list is non-empty.
- Preserve the existing empty-allocation behavior for generated-only materialization.
- Add Rust and Python source contracts that cover both the native dedup and report-reserve layers.

## Deterministic Performance Evidence

The standalone optimized Rust model projects 8,192 valid package rows through the same copied-path
vector and directory dedup set for 17 alternating samples. Filesystem copying, inventory scans,
and report writes are intentionally excluded. Both paths produced checksum `16384`; the table
records the more conservative complete run.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Container allocation calls | 25 | 2 | 92.000% |
| Requested allocation bytes | 426,124 | 213,008 | 50.013% |
| Projection P50 | 0.7147 ms | 0.2975 ms | 58.374% |
| Projection P95 | 0.8251 ms | 0.4319 ms | 47.655% |

Evidence marker: `RUNTIME07_PREALLOCATED_NATIVE_PACKAGE_REPORTS_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_preallocated_native_package_reports_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- A Rust source contract covers both materialize/preview HashSet capacities and both report
  reserves.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks passed.
- Managed Rust compilation and focused tests remain pending in a later asynchronous Runtime07
  batch; this candidate will not be validated alone.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
