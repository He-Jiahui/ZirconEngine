---
related_code:
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/naming_boundary
plan_sources:
  - docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
output_records:
  - docs/plans/zircon_runtime/runtime/05/2026-07-09-scene-editor-boundary-closeout-output-records.md
  - docs/plans/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md
---

# Runtime Naming-Boundary Current Reconciliation

Date: 2026-07-10

The existing default-feature runtime test binary ran `naming_boundary` as 98 tests: 43 passed and 55 failed. The failures were dominated by stale test infrastructure: route-only parent plans used as concrete evidence, retired aggregate status files, and production scans counting embedded test fixtures.

Current-source verification now passes:

- Runtime 15 M2 naming owners: 44/44.
- Top-level production classification and route-owner guards: 4/4.
- Folder-backed expected-slice naming/status guards: 26/26.

The production policy was not weakened. Generic module names remain forbidden in production, while test-support directories are excluded from the production-only scan. Embedded `#[cfg(test)] mod tests` lines are excluded from editor/legacy production classification. Runtime-profile `InitLevel::Editor` is explicitly classified as an editor-host target. Added evidence and lexical helpers were split into focused child owners to preserve line budgets.

A fresh full runtime test binary is still required before the complete `naming_boundary` Cargo filter can be accepted.
