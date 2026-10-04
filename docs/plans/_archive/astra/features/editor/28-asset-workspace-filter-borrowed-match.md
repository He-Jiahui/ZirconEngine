---
related_code:
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/workbench/project/asset_workspace_state/performance_tests.rs
implementation_files:
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/04/2026-09-10-asset-filter-borrowed-match.md
  - docs/plans/optimize/zircon_editor/248-editor-asset-workspace-catalog-provider-preview-import-reimport-current-working-tree-review.md
tests:
  - zircon_editor/src/ui/workbench/project/asset_workspace_state/performance_tests.rs
  - tools/tests/test_editor09_asset_catalog_generation_contract.py
  - tools/tests/test_editor_asset_browser_projection_complexity_contract.py
  - tools/tests/test_editor_asset_refresh_invalidation_authority_performance_contract.py
doc_type: milestone-detail
status: implemented_pending_validation
---

# Asset Workspace Borrowed Filter Matching

The Asset Browser workspace now performs ASCII case-insensitive matching directly over borrowed
bytes. Folder and asset filters no longer allocate lowercase copies for each candidate while
preserving the previous normalized-query semantics and field order. Folder-tree materialization
also reserves the known catalog size for its parent grouping map and flattened output vector.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor04/M28 | Remove per-candidate lowercase allocations and reserve folder-tree projection capacity | implemented_pending_validation | Combined Runtime UI/Editor asset and evidence-contract batch `111/111`; Python syntax, Rustfmt, and scoped diff checks pass. Managed Editor Cargo and Windows Release CPU/allocation/RSS p50/p95/p99 remain pending. |
