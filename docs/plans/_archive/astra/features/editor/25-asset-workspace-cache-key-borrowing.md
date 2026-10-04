---
related_code:
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/workbench/project/asset_workspace_state/performance_tests.rs
implementation_files:
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
tests:
  - zircon_editor/src/ui/workbench/project/asset_workspace_state/performance_tests.rs
  - tools/tests/test_editor09_asset_catalog_generation_contract.py
  - tools/tests/test_editor_asset_browser_projection_complexity_contract.py
  - tools/tests/test_editor_asset_event_idempotent_invalidation_performance_contract.py
  - tools/tests/test_editor_asset_pointer_generation_pressure.py
  - tools/tests/test_editor_asset_refresh_invalidation_authority_performance_contract.py
doc_type: milestone-detail
status: implemented_pending_validation
---

# Asset Workspace Borrowed Cache Key

The Asset Browser visible-item projection cache now compares its existing key against borrowed
workspace state before rebuilding an owned key. On a stable cache hit, this avoids copying the
selected-folder and search-query strings solely for equality. A cache miss still constructs the
same owned four-field key before publishing a new immutable item generation.

Catalog and resource delta patches also reserve their replacement buffers from the bounded changed
UUID/locator slice. This avoids geometric buffer growth during a multi-item Watcher refresh while
preserving the existing fallback to a full projection when an item cannot be patched locally.

The key continues to include projection generation, selected folder, search query, and kind
filter. Item ordering, filtering, selection projection, and cache invalidation behavior are
unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor04/Asset Browser cache-hit | Compare the visible-item cache key by borrowed state and reserve bounded delta replacement buffers | implemented_pending_validation | Combined Editor asset projection/Watcher static batch `52/52`, Python syntax, scoped Rustfmt, and scoped diff checks pass. Rust regressions cover every key component, borrow-before-clone ordering, and both bounded replacement reservations. Managed Editor Cargo and release CPU/allocation/frame p50/p95/p99 evidence remain pending. |
