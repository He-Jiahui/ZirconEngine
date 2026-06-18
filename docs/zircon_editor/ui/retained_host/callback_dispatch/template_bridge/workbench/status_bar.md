---
related_code:
  - zircon_editor/src/ui/workbench/snapshot/data/status_task_progress_snapshot.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_data_snapshot.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_chrome_snapshot.rs
  - zircon_editor/src/ui/workbench/model/status_bar_model.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/status_bar.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/data_sync.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/workbench.rs
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
  - zircon_editor/src/ui/retained_host/host_contract/painter/workbench.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions.rs
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_status_bar.zui
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
tests:
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/status_bar.rs
  - zircon_editor/src/tests/workbench/view_model/shell_projection.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/workbench.rs
  - cargo test -p zircon_editor --lib status_bar --locked -- --nocapture
  - cargo test -p zircon_editor --lib componentized_workbench_status_bar_skips_legacy_skeleton_fill --locked -- --nocapture
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

When `HostWindowPresentationData.workbench_window_nodes` is populated, the native painter treats the status bar as owned by the componentized `WorkbenchStatusBar` template. `draw_root_skeleton` keeps the rest of the shell fallback but skips the legacy `STATUS_BAR` quad and the old `host_shell.status_secondary` label marker for that region. Non-componentized retained windows still draw the legacy status bar fallback until the full shell cutover removes the rest of the skeleton path.

## Validation

- `componentized_workbench_status_bar_syncs_chrome_and_task_progress` verifies status text, grid/snap labels, task visibility, task text, and projected progress percent.
- `componentized_workbench_status_bar_collapses_task_slot_when_idle` verifies idle chrome collapses the task slot and removes it from host contract projection.
- `workbench_view_model_projects_status_task_progress_slot` verifies the model carries task progress and message count.
- `desktop_export_job_snapshot_projects_status_bar_task_progress` verifies desktop export job snapshots become status bar task progress records.
- `componentized_workbench_status_bar_skips_legacy_skeleton_fill` verifies the command stream no longer contains the legacy status-bar skeleton fill when the componentized Workbench window is present.
