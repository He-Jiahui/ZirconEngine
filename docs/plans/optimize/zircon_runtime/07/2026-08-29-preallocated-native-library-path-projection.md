---
title: Runtime07 Preallocated Native Library Path Projection
category: zircon_runtime
report_id: Runtime07-preallocated-native-library-path-projection-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Preallocated Native Library Path Projection

## Scope

This slice removes excess outer-vector capacity while projecting native library paths for a
discovered package. The output can never contain more entries than the requested module-kind
slice, but the previous zero-capacity vector grew to capacity four for the common Runtime+Editor
pair.

## Change

- Preallocate the library-path output to the exact `module_kinds.len()` upper bound.
- Preserve the existing path canonicalization, distribution crate preference, stable ordering,
  and merging of module kinds that resolve to one library.
- Extend the Rust regression to cover Runtime+Editor sharing one distribution library and assert
  the returned upper-bound capacity.
- Add a Python source performance contract for exact capacity and path deduplication.

## Deterministic Performance Evidence

The standalone optimized Rust model builds 262,144 two-kind path projections per sample across
31 alternating samples. Its element layout is asserted equal to the Windows production
`(PathBuf, Vec<PluginModuleKind>)` tuple size before measurement. Allocation profiles cover
65,536 projections, and both implementations produced checksum `3679443737085039408`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per 65,536 projections | 196,608 | 196,608 | 0.000% |
| Requested allocation bytes | 14,811,136 | 7,471,104 | 49.558% |
| Run 1 projection P50 | 66.2314 ms | 65.2166 ms | 1.532% |
| Run 1 projection P95 | 106.9893 ms | 89.1290 ms | 16.694% |
| Run 2 projection P50 | 82.5095 ms | 64.3384 ms | 22.023% |
| Run 2 projection P95 | 136.8277 ms | 92.8043 ms | 32.174% |

Evidence marker: `RUNTIME07_PREALLOCATED_NATIVE_LIBRARY_PATH_PROJECTION_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_preallocated_native_library_path_projection_performance_contract.py`:
  3 passed after the pre-change contract failed 2 of 3 checks.
- The standalone Rust model checks production tuple size and equivalent projection checksums;
  two runs retained identical allocation profiles and positive P50/P95 results.
- The Rust regression verifies that Runtime+Editor still merge onto one distribution library in
  stable order while the output capacity remains the exact input bound.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot publication.
- Managed Rust compilation and focused native-loader tests remain pending in the next
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
