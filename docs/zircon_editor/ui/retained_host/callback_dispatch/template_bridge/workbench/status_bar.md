---
related_code:
  - zircon_editor/src/ui/workbench/snapshot/data/status_task_progress_snapshot.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_data_snapshot.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_chrome_snapshot.rs
  - zircon_editor/src/ui/workbench/model/status_bar_model.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/shell/template_bridges.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/status_bar.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/tests.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/status_bar.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/data_sync.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_status_bar.zui
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/status_bar.rs
  - zircon_editor/src/tests/workbench/view_model/shell_projection.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/tests.rs
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
implementation_files:
  - zircon_editor/src/ui/workbench/snapshot/data/status_task_progress_snapshot.rs
  - zircon_editor/src/ui/workbench/model/status_bar_model.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/status_bar.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/shell/template_bridges.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/status_bar.rs
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_status_bar.zui
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/16-relative-layout-and-resolution-adaptation.md
  - dev/UnrealEngine/Engine/Source/Editor/CurveEditor/Private/Tree/SCurveEditorTreeFilterStatusBar.cpp
tests:
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/status_bar.rs
  - zircon_editor/src/tests/workbench/view_model/shell_projection.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/tests.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/shell/template_bridges.rs
  - cargo test -p zircon_editor --lib status_bar --locked -- --nocapture
  - cargo test -p zircon_editor --lib componentized_workbench_status_bar_skips_legacy_skeleton_fill --locked -- --nocapture
  - cargo test -p zircon_editor --lib componentized_workbench_status_bar_adapts_primary_runtime_text_without_shrinking_fixed_controls --locked -- --exact --test-threads=1
  - cargo test -p zircon_editor --lib componentized_workbench_status_bar_prioritizes_primary_text_and_active_task_by_tier --locked -- --exact --test-threads=1
doc_type: module-detail
---

# Workbench Status Bar Sync

The componentized workbench status bar is synchronized from `EditorChromeSnapshot` through `BuiltinWorkbenchWindowTemplateSurfaceBridge::sync_from_chrome`. The sync pass updates the authored status controls before scene tree and inspector reconciliation, so one presentation refresh commits the whole workbench chrome.

`StatusTaskProgressSnapshot` is the narrow editor-side task progress contract. It lives in the workbench snapshot data layer and flows through `EditorState`, `EditorDataSnapshot`, `EditorChromeSnapshot`, `StatusBarModel`, and the retained template bridge. Runtime crates do not know about editor status bar controls.

The current status model owns:

- primary status text, sourced from `EditorState::status_line`
- error, warning, and message counter labels, with task progress contributing the first live message count
- viewport grid and snap labels derived from `SceneViewportSettings`
- a fixed zoom chip, pending a future viewport zoom scalar in the chrome snapshot
- optional task progress with task id, label, detail, percent, and tone

Desktop export jobs are the first live producer. `DesktopExportJobQueue::snapshots` is converted to a `StatusTaskProgressSnapshot` in `build_export_actions.rs`, and `RetainedEditorHost::set_status_task_progress` stores the result in `EditorEventRuntime`. Running and queued jobs use info tone; cancel-requested jobs use warning tone. Finished or emptied queues clear the task slot.

The ZUI template adds `WorkbenchStatusTaskProgress` as a collapsed right-side status slot containing `WorkbenchStatusTaskLabel` and `WorkbenchStatusTaskBar`. The bridge writes the same text and value metadata onto the parent task control and the progress child so host projection tests can verify the binding without depending on painter internals.

## Adaptive composition

The status bar uses the same relative `HorizontalBox` contract at every supported width. `WorkbenchStatusReady` is the only stretch child and therefore owns remaining space; it replaces the old anonymous stretch spacer so the semantic primary status can display the full Runtime Text message. Diagnostics, task progress, and viewport tools retain explicit fixed or bounded widths, which prevents the layout solver from silently shrinking their labels and icons.

Responsive visibility is driven by the shared workbench tier classifier rather than status-control IDs or screenshot coordinates:

- Ultra and Narrow keep the primary status plus an active task, when present.
- Regular adds Errors, Warnings, and Messages.
- Wide adds Grid, Snap, the three viewport icon actions, and Zoom.

The task composite can compress from 224 to 160 logical units. Its label and progress bar compress inside matching 132-to-100 and 84-to-52 ranges while preserving the authored 8-unit internal gap. The primary item keeps a 160-unit minimum; its acceptance test measures `Blend space opened` through the Runtime Text interface and verifies containment, sibling non-overlap, and a pinned right edge at 420, 480, 481, 640, 641, 900, 1259, and 1260 logical pixels. Those exact tier boundaries prevent a representative-width test from hiding overflow immediately after a breakpoint.

This policy follows the Unreal Curve Editor status-bar pattern of one horizontal remaining-width composition with compact auto-width status content. It does not introduce absolute window positions, a control-specific host branch, a duplicate text measurement path, or a legacy fallback surface.

When `HostWindowPresentationData.workbench_window_nodes` is populated, the native painter treats the status bar as owned by the componentized `WorkbenchStatusBar` template. `draw_root_skeleton` keeps the rest of the shell fallback but skips the legacy `STATUS_BAR` quad and the old `host_shell.status_secondary` label marker for that region. The separate generic-host projection now always loads the authored `workbench_status_bar.zui`; its former constant-enabled procedural pixel fallback has been deleted. Resize/DPI recomputation preserves the last stable frames on failure and emits typed `editor_root_template_bridge_layout` or `editor_workbench_template_bridge_layout` diagnostics instead of discarding the error.

## Validation

- `componentized_workbench_status_bar_syncs_chrome_and_task_progress` verifies status text, grid/snap labels, task visibility, task text, and projected progress percent.
- `componentized_workbench_status_bar_collapses_task_slot_when_idle` verifies idle chrome collapses the task slot and removes it from host contract projection.
- `componentized_workbench_status_bar_adapts_primary_runtime_text_without_shrinking_fixed_controls` verifies the idle primary Runtime Text reserve, fixed-control widths, non-overlap, and right-edge pinning across all tier boundaries.
- `componentized_workbench_status_bar_prioritizes_primary_text_and_active_task_by_tier` verifies task compression and responsive priority across the same boundary set.
- `workbench_view_model_projects_status_task_progress_slot` verifies the model carries task progress and message count.
- `desktop_export_job_snapshot_projects_status_bar_task_progress` verifies desktop export job snapshots become status bar task progress records.
- `componentized_workbench_status_bar_skips_legacy_skeleton_fill` verifies the command stream no longer contains the legacy status-bar skeleton fill when the componentized Workbench window is present.

The adaptive composite is implemented and its source/TOML contracts are present. The managed Windows lane produced the current 3,173-test binary, but the full suite exceeded its 60-minute limit. Exact execution then exposed and corrected a test-owner mismatch: right-edge assertions now use `EditorWorkbenchTemplateFrames::status_bar` (`WorkbenchWindowStatusBarRegion`) rather than the clipped internal component root. Current-source exact tests and screenshot acceptance remain the milestone testing stage because unrelated Editor changes currently fail compilation before this body; the three Blend Space artifacts must not be refreshed until that boundary is restored.
