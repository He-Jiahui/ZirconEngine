---
related_code:
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/watch_projection.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/04/2026-09-26-watch-projection-unique-dirty-stage.md
tests:
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/watch_projection.rs
doc_type: milestone-detail
status: implemented_pending_validation
---

# Editor04 watch projection unique dirty stage

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor04 | Stage one cloned catalog record per UUID in a repeated Runtime watcher batch | `implemented_pending_validation` | Linked optimization record defines the one-clone target. Ignored `editor04_watch_storm_dirty_stage_release_benchmark` emits `EDITOR04_WATCH_STORM_DIRTY_STAGE_BENCH_V1` for 4,096 duplicate events; dirty-stage p95 must be ≤ 70% of the former path. Managed Editor compile, watcher/catalog regressions, and Release measurements remain pending in the shared batch. |
