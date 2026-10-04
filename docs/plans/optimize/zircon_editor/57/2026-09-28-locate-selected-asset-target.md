---
title: Editor57 Locate Selected Asset Target
category: zircon_editor
date: 2026-09-28
implementation_status: partial_implemented
validation_status: managed_validation_pending
performance_status: not_measured
---

# Editor57 locate selected asset target

## Defect and bounded fix

ED57-P0-02 identifies a mismatch between the `Locate selected asset` action and its result. The retained event had no target payload, and its handler only opened `editor.assets` and requested preview refresh. It did not use the selected asset identity or navigate the shared asset workspace.

The existing selection is the target for this bounded fix. When an asset is selected, the handler reads its UUID, reuses `navigate_to_asset` to select the asset's current parent folder and retain its selection, opens `editor.assets`, and requests preview refresh. When no asset is selected, it reports the localized `asset.locate.selection_required` message and does not open the Assets view. Success status uses the localized `asset.locate.success` key.

This keeps the existing event and callback protocol. It does not claim the full ED57-P0-02 reveal contract: there is no target-qualified event or `AssetRevealReceipt`, search/filter adjustment, explicit tree expansion, scroll/focus receipt, or catalog-generation receipt. The change is a partial product behavior fix, not closure of ED57-P0-02.

## Behavior regression

`zircon_editor/src/tests/host/retained_callback_dispatch/asset/locate_selected.rs` exercises the installed `LocateSelectedAsset` template control through `BuiltinAssetSurfaceTemplateBridge` and `EventRuntimeHarness`:

- `locate_selected_asset_control_navigates_to_the_selected_asset_folder` selects the scene asset while the workspace is at `res://`, invokes the Locate control, and asserts the workspace navigates to `res://scenes`, keeps that UUID selected, projects the target row, has an `editor.assets` view instance after the action, reports localized success, requests preview refresh, and journals the Locate event.
- `locate_selected_asset_without_selection_reports_failure_without_opening_assets` invokes the same control with no selected asset and asserts the English-default status message, presentation refresh, unchanged view instances and layout, journaled action, and both English and Simplified Chinese translations.

Both tests use the actual installed template binding and the normal synchronous host event route. Managed compilation and execution are pending; Cargo was not run for this slice.

## Scope and remaining product gate

The production change reads the retained selection through a crate-local `EditorState` accessor and exposes the existing `AssetWorkspaceState` UUID accessor outside `cfg(test)`. It does not add another selection authority, mutate catalog data, or add a new event variant.

The callback preserves existing search and kind-filter settings. ED57-P0-02 still needs product work for cases where a filter, virtualized scroll position, detached/hidden view, or stale external target prevents a complete visual reveal, plus a receipt that identifies the target and catalog generation. No latency, allocation, memory, or OS-level performance gate was measured.

## Validation and preservation

The `asset_workspace_state.rs` path is in the frozen 491-path v3 source manifest and has a changed successor hash. The other four touched Rust paths and both locale catalogs are new to that manifest. The preexisting edits in `asset_event.rs` (error detail field), `editor_state_asset_workspace.rs` (documentation comments), `asset_workspace_state.rs` (projection/cache work), and both locale catalogs (script-build settings copy) were preserved. The locked `retained_asset_pointer.rs` file was not modified; the new regression uses the retained callback asset test module instead.

| Gate | State |
|---|---|
| Behavior | Two real host callback regressions authored; managed validation pending. |
| Static | Rust formatting, direct UTF-8/newline/trailing-whitespace checks, scoped tracked-path diff checks, and English/Simplified Chinese TOML catalog parsing passed. Untracked paths were covered by direct byte checks because `git diff --check` does not include them. |
| Product completion | Partial only. ED57-P0-02 reveal receipt and filter/scroll/focus behavior remain open. |
| Performance | Not measured. No Editor performance gate is claimed. |
