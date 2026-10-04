---
related_code:
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/events/click.rs
  - zircon_editor/src/ui/retained_host/app/asset_surface_pointer_state.rs
  - zircon_editor/src/ui/retained_host/app/tests/drag_sources/asset_browser.rs
  - zircon_editor/assets/i18n/en.toml
  - zircon_editor/assets/i18n/zh-CN.toml
  - docs/plans/optimize/zircon_editor/57/2026-09-28-editor57-asset-browser-double-click-activation.md
plan_sources:
  - docs/plans/optimize/zircon_editor/57-editor-asset-workspace-content-browser-folder-source-tree-selection-open-create-import-rename-move-delete-history-collection-product-integration-review.md
status: partial_implemented_pending_validation
validation_status: scoped_static_checks_passed_managed_validation_pending
---

# Editor1035 / Editor57 Asset Browser double click activation completion list

| Plan item | Source result | Evidence and remaining gate | Status |
| --- | --- | --- | --- |
| ED57-P0-03 Browser double click | A second click on the same current Browser asset within 500 ms dispatches the existing `OpenAsset` event. | Classifier tests cover UUID, locator, resource revision, catalog/context, item generation, timeout, blank reset, and independent surfaces. The retained-host regression checks a matching `OpenAsset` journal record with no error and changed=true; a second callback regression changes SearchQuery without refreshing the retained surface and checks no open is dispatched. Managed Editor validation is pending. | implemented_pending_validation |
| Exact current target admission | Both clicks must match UUID, locator, resource revision, catalog revision, view mode, and visible item identity. Folder/query are not in the pointer projection; cache-key changes rebuild item-generation identity and indirectly isolate those changes. Immediately before dispatch the host compares current view mode, item-generation identity, and row locator/revision against the clicked projection. | A stale projection after query change is covered before retained UI refresh. `OpenAsset` rechecks asset type, enabled toolkit, and command. Locator dispatch is not an atomic generation/revision receipt, and toolkit view presentation is not an activation receipt. | partial_implemented |
| Activation status localization | Stale and unavailable-target outcomes use two named i18n keys with English and Chinese values and no interpolated identifiers. | Both catalog key sets retain symmetric 541 entries; managed Editor validation remains pending. | implemented_pending_validation |
| Enter, explicit Open, context menu, reference activation | Not implemented by this slice. | No common `AssetActivationIntent` or opened/reused/unavailable/failed result receipt is added. These remain open ED57-P0-03 requirements. | product_gate_pending |
| Validation and performance | No Cargo command or performance measurement was run. | Pinned rustfmt and scoped diff checks passed; run the tests with the managed Editor batch. Product latency and 100k/1M gates remain pending. | validation_pending |

The 500 ms gesture threshold follows the existing hierarchy rename double click convention. This slice
connects the retained Browser gesture to the already-authoritative asset toolkit open event; it does
not close ED57-P0-03.
