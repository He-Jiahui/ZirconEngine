---
title: Runtime07 Completed Owner Feature Selection Fast Path
category: zircon_runtime
report_id: Runtime07-completed-owner-feature-selection-fast-path-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Completed Owner Feature Selection Fast Path

## Scope

This slice removes redundant catalog projection when an owner feature selection has already been
completed. It preserves the existing merge policy for enabled, required, packaging, runtime/editor
crate names, target modes, and external provider package ownership.

## Change

- Borrow the feature identifier to resolve the existing selection index before creating an owned
  catalog projection.
- Return immediately when every field that the merge can fill is already populated.
- Require an existing provider package only when the caller supplies an external provider, so the
  incomplete-provider path still projects and merges the requested package identifier.
- Reuse the resolved index in the merge path instead of looking up the newly allocated catalog ID.
- Add Rust regressions for the completed and provider-incomplete guards plus a Python source
  performance contract for the ordering and field-completeness rules.

## Deterministic Performance Evidence

The standalone optimized Rust model repeats completion of an already populated owner selection
131,072 times per sample across 31 alternating samples. It also asserts new/old equality for
missing runtime crate, missing editor crate and target modes, and missing provider scenarios. Both
implementations produced checksum `14504336466342103191`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per 32,768 completions | 163,840 | 0 | 100.000% |
| Requested allocation bytes | 3,866,624 | 0 | 100.000% |
| Run 1 completion P50 | 61.2701 ms | 0.6346 ms | 98.964% |
| Run 1 completion P95 | 106.2736 ms | 1.1002 ms | 98.965% |
| Run 2 completion P50 | 57.9208 ms | 0.6173 ms | 98.934% |
| Run 2 completion P95 | 101.2302 ms | 0.9482 ms | 99.063% |

Evidence marker: `RUNTIME07_COMPLETED_OWNER_FEATURE_SELECTION_FAST_PATH_MODEL_V1`.

## Validation

- `python tools/tests/test_runtime07_completed_owner_feature_selection_fast_path_performance_contract.py`:
  3 passed after the pre-change contract had 3 unmet checks.
- The standalone Rust model asserts incomplete-path result equality before recording metrics; two
  runs retained identical allocation profiles, checksums, and positive P50/P95 results.
- Rust regressions cover an already complete selection and the external-provider requirement.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot publication.
- Managed Rust compilation and focused catalog tests remain pending in the next asynchronous
  Runtime07 validation batch.

Managed batch request: `runtime07-native-vm-six-task-batch-20260830-v1`.

Validation attempt: ticket `167f127a7c8d48b3a68554a5c4f1d0f7` failed during coordinator
materialization with `unmanaged_artifacts_detected` for
`D:\ZirconBuilds\mvp-test-fixtures-36724`; Cargo did not start, so integrated Rust and performance
acceptance remain pending.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
