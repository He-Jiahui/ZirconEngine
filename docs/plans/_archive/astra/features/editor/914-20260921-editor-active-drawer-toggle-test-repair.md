---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-09-21-drawer-toggle-direct-query.md
  - docs/plans/optimize/zircon_editor/135-editor-layout-profile-workspace-state-docking-tab-window-restore-schema-migration-current-source-review.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/tests/host/retained_callback_dispatch/layout/drawer_toggle.rs
---

# Editor914 Active Drawer Toggle Test Repair

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Hierarchy drawer collapse/reopen | Read all three before/after drawer states through `WorkbenchLayout::active_activity_window_drawers()` rather than the retired root `drawers` field, retaining pinned/collapsed modes, active instance identity, journaled layout events, and dirty-effect assertions. | v27 Editor check reported three missing root fields in this clean test owner. Local Rustfmt and diff checks pass; the foreign-modified layout source is untouched. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/tests/host/retained_callback_dispatch/layout/drawer_toggle.rs` | `F83B904A84D2833AB996ECC123820CB287D7C6CDFB92160D91FDD639FDF6ABFC` |

## Managed gate

This test repair postdates v28 admission. The next grouped current-source
Editor Rust regression must verify both modes; the direct-query ignored
Release, allocation, and product percentile gates remain pending.
