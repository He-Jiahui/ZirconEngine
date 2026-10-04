---
related_code:
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh/normalize.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh/reconcile.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/359/2026-09-26-reconcile-borrowed-asset-id-admission.md
tests:
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh/normalize.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh/reconcile.rs
doc_type: milestone-detail
status: implemented_pending_validation
---

# Editor359 reconcile borrowed asset IDs

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor359 | Reuse borrowed normalized asset-ID admission during bounded UI Asset watcher reconciliation | `implemented_pending_validation` | Fragment-alias behavior regression and allocation target are recorded. `EDITOR359_BORROWED_UI_ASSET_CHANGE_DEDUP_BENCH_V1` retains its pending duplicate-heavy p95 ≤ 70% target; `EDITOR359_UI_ASSET_CHANGE_UNIQUE_MIXED_BENCH_V1` adds pending all-unique and 50%-unique p95 ≤ 110% non-regression gates. Managed Editor check, Release results, and product overflow percentiles await the combined batch. |
