---
related_code:
  - zircon_editor/src/ui/layouts/views/asset_browser/toolbar_layout.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/toolbar_layout/control_snapshot.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/toolbar_layout/control_snapshot_tests.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/57/2026-09-26-active-dropdown-toolbar-control-snapshot.md
tests:
  - zircon_editor/src/ui/layouts/views/asset_browser/toolbar_layout/control_snapshot_tests.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/toolbar_responsiveness_tests.rs
doc_type: milestone-detail
status: implemented_pending_validation
---

# Editor57 active dropdown toolbar control snapshot

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor57 | Read seven toolbar controls in one pass on the current Asset Browser dropdown path, preserving first-match and width fallback semantics | `implemented_pending_validation` | Behavior regression, actual-template toolbar regressions, and `EDITOR57_ACTIVE_DROPDOWN_TOOLBAR_READ_BENCH_V1` are queued for the combined Editor batch. Read-projection p95 targets are ≤ 110% on a small node table and ≤ 70% on a large node table; full layout and product latency remain unmeasured. |
