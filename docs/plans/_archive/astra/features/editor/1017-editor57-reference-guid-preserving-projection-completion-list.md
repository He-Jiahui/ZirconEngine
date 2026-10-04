---
related_code:
  - zircon_editor/src/ui/host/editor_asset_manager/manager/catalog_generation/details.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/catalog_generation/record.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/57/2026-09-27-reference-guid-preserving-projection.md
tests:
  - zircon_editor/src/ui/host/editor_asset_manager/manager/catalog_generation/record/reference_identity_tests.rs
doc_type: milestone-detail
status: implemented_pending_validation
---

# Editor57 reference GUID preserving projection completion list

Independent source/API review, scoped Rustfmt, whitespace, and preimage
preservation checks passed. Four real test functions remain unexecuted pending
the managed Runtime/Editor batch; no product performance pass is claimed.

| Work | Source evidence | Remaining acceptance |
|---|---|---|
| Preserve missing explicit UUIDs when another asset occupies the old locator. | Details and catalog rows share one private identity lookup; unresolved rows retain the original UUID/locator and are not known-project pointer targets. Existing foreign capacity changes are retained. | Grouped managed compile and `editor57_reference_identity_` regressions pending. |
| Preserve legitimate locator-only behavior and UUID priority. | Only the exact `AssetReference::from_locator` identity may fall back; an already registered UUID always wins. Runtime strict registry semantics are unchanged. | Managed behavior validation pending. |
| Verify the consumer route. | Real details/catalog projection feeds `AssetWorkspaceState` and a reference pointer click; explicit missing identity does not navigate to the occupying asset, while valid targets navigate correctly. | Native product qualification and Editor57-G37 p95/p99 remain pending; no new timing result is claimed. |
