Plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
Milestone: M3
Status: pending
Files:
  - docs/plans/zircon_editor/editor/05/2026-07-31-m3-hierarchy-current-source-manifest.md
  - zircon_editor/assets/ui/editor/hierarchy.zui
  - zircon_editor/src/core/editing/intent.rs
  - zircon_editor/src/core/editor_event/hierarchy_host_event.rs
  - zircon_editor/src/core/editor_event/mod.rs
  - zircon_editor/src/core/editor_event/types.rs
  - zircon_editor/src/tests/editing/node_ops.rs
  - zircon_editor/src/ui/host/editor_event_execution/dispatch.rs
  - zircon_editor/src/ui/host/editor_event_execution/hierarchy_event.rs
  - zircon_editor/src/ui/host/editor_event_execution/mod.rs
  - zircon_editor/src/ui/host/editor_event_execution/undo_policy.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_filter.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/drag_source.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events/click.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events/drag.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_rename.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/pointer_surfaces.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/presentation.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/state/construction/assembly.rs
  - zircon_editor/src/ui/retained_host/app/native_keyboard_actions.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/edit.rs
  - zircon_editor/src/ui/retained_host/app/tests/drag_sources.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/hierarchy/edit.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/hierarchy/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/content.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/content.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/hierarchy.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/hierarchy/row.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/hierarchy/row/text.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation/pane_conversion.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation/scene_conversion.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/hierarchy_projection.rs
  - zircon_editor/src/ui/workbench/state/editor_state_apply_intent.rs
---

# Editor05 M3 Hierarchy Current-Source Manifest

## Scope Delivered

The M3.1 candidate routes hierarchy reparenting, inline rename, multi-selection deletion, and search filtering through typed editor events and transaction-backed intent handling. It preserves authoritative selection for keyboard and drag actions, collapses nested selections to top-level reparent roots, and keeps Unicode case-folding behavior when the ASCII fast path is not applicable.

## Fresh Testing Evidence

- Rust 1.94.1 scoped `rustfmt --check` passed for the exact M3 Rust source scope; the final `hierarchy_filter.rs` change was formatted independently after the profile-boundary and deep-hierarchy regressions were added.
- Static whitespace checks passed for the current hierarchy-filter source. Managed Cargo validation has not yet been requested through this candidate manifest.
- The current source includes flat and deep 5,000-entry hierarchy correctness regressions. The plan's host-level latency evidence remains a managed `UiPerfScenario::Click` task; no wall-clock unit-test result is claimed.

## Review

The first post-profile review found an Important deep-hierarchy all-match O(N^2) ancestor-propagation path. The current source replaces it with parent indexing plus one reverse propagation pass. A subsequent static audit found and repaired the deep-test `u64` node-ID versus `usize` depth type mismatch. The final compile-aware independent review reported Critical/Important/Minor = 0/0/0, covering explicit ID/depth types, pre-order parent indexing, ancestor inclusion, O(N + total name characters) behavior, the production-only profile-scope guard, and ASCII/Unicode filtering behavior. Prior reviewed coverage includes authoritative selection for drag and F2 and nested reparent roots.

## 产出记录与时间

No accepted output: M3 remains pending source-bound managed Cargo validation and the Editor02 M2 diff-refresh dependency. Any pre-fix coordinator request is not current-source acceptance evidence.
