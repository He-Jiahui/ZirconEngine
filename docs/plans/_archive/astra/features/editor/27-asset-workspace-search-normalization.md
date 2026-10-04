---
related_code:
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/workbench/project/asset_workspace_state/performance_tests.rs
implementation_files:
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/04/2026-09-10-search-query-normalization-cache.md
tests:
  - zircon_editor/src/ui/workbench/project/asset_workspace_state/performance_tests.rs
  - tools/tests/test_editor09_asset_catalog_generation_contract.py
  - tools/tests/test_editor_asset_browser_projection_complexity_contract.py
  - tools/tests/test_editor_asset_event_idempotent_invalidation_performance_contract.py
  - tools/tests/test_editor_asset_pointer_generation_pressure.py
  - tools/tests/test_editor_asset_refresh_invalidation_authority_performance_contract.py
  - tools/tests/test_editor_ui_asset_workspace_watcher_performance_contract.py
  - tools/tests/test_editor09_ui_asset_watcher_bounded_refresh_contract.py
  - tools/tests/test_editor09_ui_asset_watcher_generation_contract.py
doc_type: milestone-detail
status: implemented_pending_validation
---

# Asset Workspace Search Normalization Cache

`AssetWorkspaceState` now caches the normalized Asset Browser search string at the setter boundary.
Stable snapshots and catalog watcher patches borrow the cached `&str` instead of allocating a
lowercase `String` on every call. ASCII case-insensitive matching and all projection generation
and invalidation behavior remain unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M27 | Cache search normalization across stable workspace snapshots and catalog delta patches | implemented_pending_validation | Editor asset projection/Watcher static batch `52/52`; Python syntax, scoped Rustfmt, and scoped diff checks pass. Rust regressions cover mixed-case and idempotent setter behavior plus both no-per-call-normalization guards. Managed Editor Cargo and Windows Release allocation/time p50/p95/p99 evidence remain pending. |
