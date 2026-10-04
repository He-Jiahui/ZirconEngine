---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/04/2026-09-20-material-projection-row-capacity.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/material_editor/projection.rs
  - zircon_editor/src/ui/material_editor/projection/row_capacity_tests.rs
tests:
  - tools/tests/test_editor_material_projection_row_capacity_performance_contract.py
---

# Editor856 - material projection row capacity

## Completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Material property rows | Reserve the shader-property plus material-override upper bound before projecting rows, preserving declared order and unknown-override append order. | TDD source/model contract `4/4` after a RED run with two structural failures and one missing lower owner; lower order/capacity regression and ignored `EDITOR856_MATERIAL_PROJECTION_ROW_CAPACITY_BENCH_V1` marker are wired. The dense 8,192-row model changes modeled growth events `12→0`; the performance/pressure loader passes `2424/2424` across `659` files in `52.778s`, and the current expanded source-contract loader passes `4072/4072` across `962` files in `139.499s`; managed Cargo/Release, allocator, and Material Editor product p50/p95/p99 evidence remain pending. | implemented_pending_validation |
| Material texture rows | Reserve the shader-texture plus material-texture upper bound with saturating addition, preserving duplicate suppression, row payloads, and empty behavior. | Same focused contract and lower owner; no diagnostic projection or Editor624 membership semantics changed. | implemented_pending_validation |

## Complexity boundary

This slice changes only the two row-vector reservation sites in the material
editor projection. It does not alter shader validation, diagnostic ordering,
material authority, editor commands, runtime ABI, or tooling production.

## Managed gate

No managed Windows Cargo/Release command is started locally and coordinator
status is not polled. Keep this entry `implemented_pending_validation` until
the combined owner-attributed Windows Release lane proves current-source
compilation, lower test reachability, allocator behavior, and Material Editor
product p50/p95/p99 evidence.
