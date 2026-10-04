---
related_code:
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/workbench/project/asset_workspace_state/performance_tests.rs
implementation_files:
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/04/2026-09-10-borrowed-folder-membership.md
  - docs/plans/optimize/zircon_editor/248-editor-asset-workspace-catalog-provider-preview-import-reimport-current-working-tree-review.md
tests:
  - zircon_editor/src/ui/workbench/project/asset_workspace_state/performance_tests.rs
  - tools/tests/test_editor09_asset_catalog_generation_contract.py
  - tools/tests/test_editor_asset_browser_projection_complexity_contract.py
  - tools/tests/test_editor_asset_refresh_invalidation_authority_performance_contract.py
doc_type: milestone-detail
status: implemented_pending_validation
---

# Asset Workspace Borrowed Folder Matching

The Asset Browser now checks an asset's parent folder through borrowed locator slices during
full projection and catalog delta patching. Existing owned parent-ID materialization remains
available for navigation callers, while the repeated visibility predicate no longer allocates a
temporary `String` per candidate.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor04/M29 | Remove per-candidate owned parent-folder materialization from asset visibility and patch predicates | implemented_pending_validation | Combined Runtime/Editor hotpath contract batch `135/135` passed; focused asset/Taffy recheck `53/53` also passed, including UTF-8 boundary equivalence; Python syntax, Rustfmt, source guard, and scoped diff checks pass. Managed Editor Cargo and Windows Release CPU/allocation/RSS p50/p95/p99 remain pending. |
