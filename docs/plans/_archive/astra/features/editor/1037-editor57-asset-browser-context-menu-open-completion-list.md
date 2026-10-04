---
related_code:
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/context_menu.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/context_menu.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/context_menu.rs
  - docs/plans/optimize/zircon_editor/57/2026-09-28-editor57-asset-browser-context-menu-open.md
plan_sources:
  - docs/plans/astra/features/editor/1035-editor57-asset-browser-double-click-activation-completion-list.md
status: partial_implemented_pending_validation
validation_status: scoped_static_checks_passed_managed_validation_pending
---

# Editor1037 / Editor57 Asset Browser context menu Open completion list

| Plan item | Source result | Evidence and remaining gate | Status |
| --- | --- | --- | --- |
| Context menu Open action | The retained Asset Browser row menu now exposes localized Open and routes its typed binding through `AssetCommand::OpenAsset`. | Bridge regression asserts a supplied encoded locator is preserved in the binding; host menu encoding was statically reviewed, without a real right-click construction test. The binding does not read current selection, so a later selection of B cannot replace context target A. The Delete regression checks UUID routing when the path also contains an Open locator. Cargo execution is pending. | implemented_pending_validation |
| Stale or unavailable context target | The menu dispatch carries the exact locator captured for its row. | Existing `OpenAsset` remains responsible for resolving stale/unavailable locators and toolkit availability. No atomic catalog-generation fence is added here. | partial_implemented |
| Common activation contract | Not implemented by this slice. | Enter activation, `AssetActivationIntent`, and opened/reused/unavailable/failed receipts remain open. Native toolkit presentation is not confirmed by a dispatch event. | product_gate_pending |
| Validation and performance | No Cargo command or performance measurement was run. | Pinned rustfmt and scoped static checks passed; managed editor validation and product performance gates remain pending. | validation_pending |

This context-menu route is a partial ED57-P0-03 slice and does not close the activation plan.
