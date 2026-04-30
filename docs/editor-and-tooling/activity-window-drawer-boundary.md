---
related_code:
  - zircon_editor/src/ui/workbench/layout/activity_window_id.rs
  - zircon_editor/src/ui/workbench/layout/activity_window_host_mode.rs
  - zircon_editor/src/ui/workbench/layout/activity_window_layout.rs
  - zircon_editor/src/ui/workbench/layout/main_host_page_layout.rs
  - zircon_editor/src/ui/workbench/layout/editor_main_frame_layout.rs
  - zircon_editor/src/ui/workbench/layout/workbench_layout.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_chrome_snapshot_build.rs
  - zircon_editor/src/ui/workbench/model/build/workbench_view_model_build.rs
  - zircon_editor/src/ui/workbench/autolayout/region/tool_region/presence.rs
  - zircon_editor/src/ui/workbench/view/activity_window_template_spec.rs
  - zircon_editor/src/ui/workbench/view/view_descriptor.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/assets/ui/editor/host/editor_main_frame.ui.toml
  - zircon_editor/assets/ui/editor/host/activity_drawer_window.ui.toml
  - zircon_editor/assets/ui/editor/windows/workbench_window.ui.toml
  - zircon_editor/assets/ui/editor/windows/asset_window.ui.toml
  - zircon_editor/assets/ui/editor/windows/ui_layout_editor_window.ui.toml
implementation_files:
  - zircon_editor/src/ui/workbench/layout/activity_window_layout.rs
  - zircon_editor/src/ui/workbench/layout/activity_window_id.rs
  - zircon_editor/src/ui/workbench/layout/main_host_page_layout.rs
  - zircon_editor/src/ui/workbench/layout/editor_main_frame_layout.rs
  - zircon_editor/src/ui/workbench/layout/workbench_layout.rs
  - zircon_editor/src/ui/workbench/layout/manager/apply.rs
  - zircon_editor/src/ui/workbench/layout/manager/attach.rs
  - zircon_editor/src/ui/workbench/layout/manager/focus.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_chrome_snapshot_build.rs
  - zircon_editor/src/ui/workbench/snapshot/workbench/main_page_snapshot.rs
  - zircon_editor/src/ui/workbench/model/build/workbench_view_model_build.rs
  - zircon_editor/src/ui/workbench/autolayout/region/tool_region/presence.rs
  - zircon_editor/src/ui/host/builtin_layout/hybrid_layout.rs
  - zircon_editor/src/ui/host/builtin_layout/workbench_page.rs
  - zircon_editor/src/ui/workbench/view/activity_window_template_spec.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/assets/ui/editor/host/editor_main_frame.ui.toml
  - zircon_editor/assets/ui/editor/host/activity_drawer_window.ui.toml
  - zircon_editor/assets/ui/editor/windows/workbench_window.ui.toml
  - zircon_editor/assets/ui/editor/windows/asset_window.ui.toml
  - zircon_editor/assets/ui/editor/windows/ui_layout_editor_window.ui.toml
plan_sources:
  - user: 2026-04-25 ActivityDrawer should move into ActivityWindow; main frame keeps only task bar and window tabs; no Slint business UI
  - docs/superpowers/specs/2026-04-25-editor-activity-window-design.md
  - docs/superpowers/plans/2026-04-25-editor-activity-window-restructure.md
tests:
  - zircon_editor/src/tests/workbench/layout/activity_window_layout.rs
  - zircon_editor/src/tests/workbench/layout/editor_main_frame_layout.rs
  - zircon_editor/src/tests/workbench/layout/window_drawer_ownership.rs
  - zircon_editor/src/tests/workbench/view_model/shell_projection.rs
  - zircon_editor/src/tests/host/builtin_window_descriptors.rs
  - zircon_editor/src/tests/host/template_runtime/pane_body_documents.rs
  - zircon_editor/tests/editor_main_frame_template.rs
  - zircon_editor/tests/activity_drawer_window_template.rs
  - zircon_editor/tests/workbench_window_template.rs
  - zircon_editor/tests/asset_window_template.rs
  - zircon_editor/tests/ui_layout_editor_window_template.rs
  - cargo test -p zircon_editor --lib workbench_view_model_uses_only_active_activity_window_drawers --locked --no-run (blocked 2026-04-29 by unrelated zircon_runtime Hybrid GI source-type compile errors)
  - cargo fmt --package zircon_editor -- --check
doc_type: module-detail
---

# Activity Window Drawer Boundary

The editor main frame is now modeled separately from drawer-capable editor windows. `EditorMainFrameLayout` stores only the active window and ordered window tabs. Drawer-capable windows use `ActivityWindowLayout`, which owns `activity_drawers` and a content workspace.

The reusable `.ui.toml` shell is `editor.host.activity_drawer_window`. `WorkbenchWindow`, `AssetWindow`, and `UILayoutEditorWindow` reference that shell and mount their own left, right, bottom, and content regions. Slint remains a host projection layer; these new business window structures are `.ui.toml` assets.

`WorkbenchLayout::activity_windows()` remains the compatibility read surface for old layouts that still only persisted root `drawers`, but current presentation no longer treats the first or root drawer map as active by default. `MainHostPageLayout::WorkbenchPage` now binds the page to an `ActivityWindowId`, and `EditorChromeSnapshot::build(...)` projects drawer snapshots only from the activity window attached to `active_main_page`.

The view model and autolayout path consume that active-window drawer snapshot directly. `WorkbenchViewModel.drawer_ring` is hidden and empty when the active window has no configured drawers, so switching from `WorkbenchWindow` to an `AssetBrowserWindow` with no drawer configuration no longer preserves the previous left/right/bottom edge drawers. Drawer extent calculation also reads the active window drawer snapshots instead of the legacy root `WorkbenchLayout.drawers` map, preventing stale Workbench drawer widths from sizing a different activity window.

Drawer mutation commands (`OpenView` into a drawer, `SetDrawerMode`, `SetDrawerExtent`, and `ActivateDrawerTab`) now resolve through `WorkbenchLayout::active_activity_window_mut()`. Focusing a drawer tab records the owning activity window and re-activates the page bound to that window. This keeps layout state, snapshot projection, and host geometry aligned around the same ownership rule: the current `ActivityWindowLayout` decides whether left, right, and bottom drawers exist and how large they are.
