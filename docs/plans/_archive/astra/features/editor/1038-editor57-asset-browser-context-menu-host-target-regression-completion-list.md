---
related_code:
  - zircon_editor/src/ui/retained_host/app/tests/drag_sources/asset_browser.rs
  - docs/plans/optimize/zircon_editor/57/2026-09-28-editor57-asset-browser-context-menu-host-target-regression.md
plan_sources:
  - docs/plans/astra/features/editor/1037-editor57-asset-browser-context-menu-open-completion-list.md
status: test_added_pending_validation
validation_status: scoped_static_checks_passed_managed_validation_pending
---

# Editor1038 / Editor57 Asset Browser context menu target regression completion list

| Plan item | Source result | Evidence and remaining gate | Status |
| --- | --- | --- | --- |
| Retained host right-click target | A real Browser right-button callback opens the context menu from row A while asset B is selected. | The regression checks retained `context_target_path`, obtains the actual Open binding, and verifies typed asset dispatch carries A's locator while selection remains B. Cargo execution is pending. | test_added_pending_validation |
| Toolkit execution and presentation | Not covered by this regression. | Test evidence ends at `AssetHostEvent::OpenAsset`; toolkit resolution and native presentation remain separate gates. | product_gate_pending |
| ED57-P0-03 completion | Not complete. | Enter activation, common `AssetActivationIntent` and opened/reused/unavailable/failed receipt, atomic catalog-generation validation, and product performance gates remain open. | product_gate_pending |

No Cargo validation or performance measurement was run for this slice.
