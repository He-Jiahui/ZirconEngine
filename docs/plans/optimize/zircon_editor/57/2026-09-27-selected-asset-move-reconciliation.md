---
title: Editor57 Selected Asset Move Reconciliation
category: zircon_editor
date: 2026-09-27
implementation_status: implemented
validation_status: batched_validation_pending
performance_status: product_gate_pending
---

# Editor57 selected asset move reconciliation

## Defect and bounded contract

ED57-P1-14 and the folder/list/details portion of ED57-G14 identify a catalog reconciliation defect: when an external move keeps the selected asset UUID, the previous code keeps the old `selected_folder_id` while the selection projection reads the new locator. The asset then disappears from the content list for the old folder even though details refer to its new location.

Both full and exact-change catalog synchronization now compare the old and new selected asset rows by indexed UUID lookup before replacing the catalog. If the locator changes to another existing parent folder and the browsed folder still equals the previous parent, the selected folder follows that parent and the UUID remains selected. A rename inside the same folder keeps the folder. A locator change invalidates the previously retained details generation, which may describe the old location; normal manager detail refresh must publish current details again. An unchanged locator or unrelated asset update does not navigate. A detached selection preserves its independently browsed folder even when the selected asset moves, while its selection projection reads the new locator and drops old details. Asset deletion still clears selection/details, and a missing selected folder still uses the existing root fallback. A new parent absent from the catalog is not adopted.

This is catalog reconciliation, not a user navigation event. It does not append history, synthesize focus or reveal receipts, clear query/kind filters, or change either surface's view and utility preferences. ED57-G14's focus/history/atomic receipt portion remains pending.

## Regression evidence and input

`zircon_editor/src/tests/editing/asset_workspace.rs` adds behavior regressions through the production `AssetWorkspaceState` methods:

- `selected_asset_external_move_reconciles_folder_in_full_and_exact_catalog_sync` warms the material folder generation, moves the same UUID to the existing scenes folder, rebuilds the complete catalog record with source/destination membership and locator indices, and exercises both synchronization entry points. It asserts folder, content item, selected UUID, new locator, selected source-tree row, query/kind preservation, separate surface preferences, old-details invalidation and stable post-move item-generation reuse.
- `unchanged_selected_locator_and_unrelated_delta_preserve_folder_and_item_generation` checks both unchanged sync entry points and a rebuilt catalog with an unrelated exact delta. The selected folder remains unchanged and the immutable visible-item generation is reused.
- `catalog_sync_does_not_navigate_an_unchanged_selection_from_another_folder` preserves a deliberately detached selection without changing its browsed folder.
- `selected_asset_external_move_preserves_detached_folder_in_full_and_exact_catalog_sync` browses textures, selects the material UUID, then rebuilds the complete material-to-scenes move catalog. Both sync entry points and both surface projections retain the textures folder and its content, retain the selected UUID, project its new locator and discard old-locator references.
- `selected_asset_rename_invalidates_old_details_without_changing_folder` checks a same-folder locator change.
- `selected_asset_removal_clears_selection_in_full_and_exact_catalog_sync` checks deletion and source-folder membership in both entry points.

The move fixture uses `EditorAssetCatalogGeneration::from_snapshot_record`, not the payload-only `updated_asset` helper, so moved locator and folder membership are consistent. The primary move assertion distinguishes the previous defect: the old implementation leaves the selected folder at `res://materials` while the asset is now at `res://scenes/grid.zmaterial`. These tests were authored before the production change; execution is pending the grouped managed Batch K receipt.

## Scope, performance and preservation

The new reconciliation performs two indexed selected-UUID lookups and no scan over catalog assets or folders. Parent-path allocation occurs only when the selected locator changes; unchanged and unrelated updates remain allocation-free in this new comparison stage. Item-generation keys already contain the selected folder, so a move triggers correct visible-item rebuilding and subsequent stable snapshots reuse the published generation.

This slice does not optimize `build_chrome()` or full folder-tree materialization. It adds no independent pure benchmark fixture and claims no 100,000-asset latency result. The integrated 100,000-item native navigation/selection/action p95/p99 baseline and ED57-G37 remain pending, as does the ED57-G14 product receipt gate.

The existing state source contained foreign query-normalization, cache-key, parent-filter, capacity and resource-generation changes. Their pre-edit diff is retained in the read-only coordinator backup `.codex/state/session-coordinator/async-validation-batches/2026-09-27-astra-optimize-batch-k-preexisting-diffs.json`. Reconstructing the source from HEAD plus that backup and reversing only this slice's two call replacements and helper insertion proved the foreign source bytes unchanged.

## Validation manifest

| Gate | Scope | State |
|---|---|---|
| Static | Both Rust source paths and this optimize/Astra record | Rustfmt and scoped `git diff --check` passed; all four paths passed UTF-8/LF, final-newline and whitespace checks. Rust compilation remains pending. |
| Behavior | Six new tests above in `zircon_editor/src/tests/editing/asset_workspace.rs` | Combined managed Batch K compile/test pending |
| Currentness/product | Current details refresh; ED57-G14 focus/history/receipt and ED57-G37 native 100,000-item p95/p99 | Pending; local state reconciliation does not certify these gates |
