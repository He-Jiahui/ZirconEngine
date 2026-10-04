---
related_code:
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/tests/editing/asset_workspace.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/57/2026-09-27-selected-asset-move-reconciliation.md
tests:
  - zircon_editor/src/tests/editing/asset_workspace.rs
doc_type: milestone-detail
status: implemented_pending_validation
---

# Editor57 selected asset move reconciliation

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor57 | Follow a selected asset's existing new parent after a catalog move preserves its UUID when still browsing its previous parent; preserve detached browsing, invalidate old-locator details and preserve filtering and surface preferences; retain unchanged/unrelated generation reuse and deletion behavior | `implemented_pending_validation` | Six complete-catalog behavior regressions authored in `asset_workspace.rs`, pending grouped managed Batch K compile/test. Foreign state source differences preserved against the coordinator pre-edit backup. ED57-P1-14 local state repair is implemented; ED57-G14 focus/history/atomic receipt, current details refresh and ED57-G37 native 100,000-item p95/p99 remain pending. |
