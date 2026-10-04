---
related_code:
  - zircon_editor/src/core/asset/mod.rs
  - zircon_editor/src/core/asset/activation.rs
  - zircon_editor/src/core/asset/activation/tests.rs
  - docs/plans/optimize/zircon_editor/57/2026-09-28-asset-activation-contract.md
plan_sources:
  - docs/plans/optimize/zircon_editor/57-editor-asset-workspace-content-browser-folder-source-tree-selection-open-create-import-rename-move-delete-history-collection-product-integration-review.md
status: contract_defined_pending_execution_and_validation
validation_status: scoped_static_checks_passed_managed_validation_pending
---

# Editor1039 / Editor57 asset activation contract completion list

| Plan item | Source result | Evidence and remaining gate | Status |
| --- | --- | --- | --- |
| Typed activation target | `AssetActivationIntent` binds typed UUID and locator to captured catalog revision, optional resource revision, and five explicit activation sources. | JSON regressions preserve both identities, large revision values, absence of a resource revision, and every source; unsupported locator decode is rejected. Managed Editor tests remain pending. | contract_defined_pending_validation |
| Explicit terminal receipt | `AssetActivationReceipt` retains the intent and has opened, reused, unavailable, and failed results with view instance ID or diagnostic text. | All four results have JSON round-trip regressions. Existing event execution does not emit these receipts, and `EditorEventResult::failure` leaves journal `value=None`. | contract_defined_execution_pending |
| ED57-P0-03 product activation | Existing double click and context menu still use locator-only `OpenAsset`. | Enter/reference/common execution, atomic UUID and revision checks, exact toolkit resolution, native presentation, managed Cargo, and 100k/1M product gates remain open. No measured performance result is claimed. | product_gate_pending |

This candidate records a core data contract only; it does not close ED57-P0-03.
