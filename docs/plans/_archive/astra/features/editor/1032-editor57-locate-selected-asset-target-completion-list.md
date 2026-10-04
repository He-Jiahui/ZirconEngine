---
related_code:
  - zircon_editor/src/ui/host/editor_event_execution/asset_event.rs
  - zircon_editor/src/ui/workbench/project/editor_state_asset_workspace.rs
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/asset/locate_selected.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/asset/mod.rs
  - zircon_editor/assets/i18n/en.toml
  - zircon_editor/assets/i18n/zh-CN.toml
plan_sources:
  - docs/plans/optimize/zircon_editor/57/2026-09-28-locate-selected-asset-target.md
status: partial_implemented_pending_validation
validation_status: scoped_static_checks_passed_managed_validation_pending
---

# Editor1032 / Editor57 locate selected asset target completion list

| Plan item | Source result | Evidence and remaining gate | Status |
| --- | --- | --- | --- |
| ED57-P0-02 selected target | Locate reads the current selected asset UUID and navigates the shared workspace to its parent folder before opening Assets. | `locate_selected_asset_control_navigates_to_the_selected_asset_folder` exercises the installed template control and asserts folder, UUID, visible target row, an `editor.assets` view instance after the action, localized success status, view effects, preview refresh, and journal event. Managed Editor validation is pending. | implemented_pending_validation |
| ED57-P0-02 no selection | An empty selection reports a localized status message and leaves the view instances and layout unchanged. | `locate_selected_asset_without_selection_reports_failure_without_opening_assets` asserts the English-default message, unchanged view instances and layout, and English/Chinese translations through the installed control. Managed Editor validation is pending. | implemented_pending_validation |
| ED57-P0-02 full reveal receipt | Not implemented by this slice. | Target-qualified intent, catalog-generation receipt, filter adjustment, explicit tree expansion, scroll/focus result, and stale/hidden-target reporting remain open. This batch does not close ED57-P0-02. | product_gate_pending |
| Managed validation and performance | No Cargo command or performance measurement was run. | Submit the focused tests through the managed Editor validation path. Allocation, latency, memory, and live UI gates remain pending. | validation_pending |

The selected-asset getter and event handler reuse the existing workspace selection and `navigate_to_asset` behavior. The two user-visible status lines resolve through `asset.locate.selection_required` and `asset.locate.success` in both locale catalogs. Existing foreign edits were preserved. `asset_workspace_state.rs` changes a v3-manifest path; the other four Rust source/test paths and both locale catalogs are new to that manifest. The locked `retained_asset_pointer.rs` path was not changed. No managed receipt or performance pass is claimed.
