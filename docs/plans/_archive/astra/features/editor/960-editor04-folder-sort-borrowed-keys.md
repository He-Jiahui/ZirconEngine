---
related_code:
  - zircon_editor/src/ui/host/editor_asset_manager/manager/catalog_generation/folders.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/04/2026-09-26-folder-sort-borrowed-keys.md
tests:
  - zircon_editor/src/ui/host/editor_asset_manager/manager/catalog_generation/folders.rs
  - zircon_editor/src/tests/editing/asset_workspace.rs
doc_type: milestone-detail
status: implemented_pending_validation
---

# Editor04 folder sort borrowed keys

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor04 | Sort sibling folder IDs directly and borrow asset display names in catalog folder generation | `implemented_pending_validation` | Source and structural allocation targets recorded in the linked optimization record. Ignored `editor04_folder_sort_and_large_catalog_release_benchmark` emits `EDITOR04_FOLDER_SORT_LARGE_CATALOG_BENCH_V1` for 4,096 sibling folders; sort p95 must be ≤ 70% of the former path, and full catalog p50/p95/p99 are reported. Managed Editor compile, focused regressions, and Release measurements remain pending in one shared batch. |
