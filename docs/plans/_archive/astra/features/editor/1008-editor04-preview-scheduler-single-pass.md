---
related_code:
  - zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/sync_from_project.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/sync_from_project/preview_scheduler_tests.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/04/2026-09-27-preview-scheduler-single-pass.md
tests:
  - zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/sync_from_project/preview_scheduler_tests.rs
doc_type: milestone-detail
status: implemented_pending_validation
---

# Editor04 preview scheduler single pass

## Plan completion list

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Editor04 | Snapshot the matching catalog generation under the editor state read lock; merge preview results and construct one scheduler after unlocking | `implemented_pending_validation` | Preview state, changed-source demand, and same-revision rebase regressions plus `EDITOR04_PREVIEW_SCHEDULER_LARGE_CATALOG_BENCH_V1` are queued for the combined Editor batch. Each size retains 31 alternating raw old/new prepare and read-lock samples after five warmups, with p50/p95/p99. The 10,000-record prepare p95 target is at most 110% of the prior stage, the 100,000-record target is at most 95%, and read-lock hold p95 is at most 10% at both sizes; Release outputs and product UI p99 remain pending. |
