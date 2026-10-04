---
related_code:
  - zircon_editor/src/lib.rs
  - zircon_editor/src/ui/host/editor_manager_asset_workspace.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/watcher/diagnostics.rs
implementation_files:
  - zircon_editor/src/lib.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
tests:
  - tools/tests/test_editor09_ui_asset_watcher_bounded_refresh_contract.py
  - zircon_editor/src/tests/host/manager/ui_asset_workspace_watcher.rs
doc_type: milestone-detail
status: implemented_pending_validation
---

# UI Asset Watcher Poll Report Export

`EditorManager::poll_ui_asset_workspace_watcher` already returned a typed, bounded poll report,
but the report type was not exposed from the crate root. The crate now re-exports both the report
and its diagnostics type, restoring a usable public contract without restoring the retired
`Vec<String>` compatibility result.

This is an API visibility repair only. It does not change watcher budgets, coalescing, reconcile
cursor behavior, or polling work.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor09/Watcher API | Re-export the typed workspace watcher report and diagnostics from the Editor crate root | implemented_pending_validation | The same combined Editor asset projection/Watcher static batch passes `52/52`, including the previously failing root-export contract; scoped Rustfmt and diff checks pass. Managed Editor Cargo remains pending. |
