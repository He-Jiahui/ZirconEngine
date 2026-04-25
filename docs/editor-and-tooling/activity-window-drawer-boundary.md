---
related_code:
  - zircon_editor/src/ui/workbench/layout/activity_window_id.rs
  - zircon_editor/src/ui/workbench/layout/activity_window_host_mode.rs
  - zircon_editor/src/ui/workbench/layout/activity_window_layout.rs
  - zircon_editor/src/ui/workbench/layout/editor_main_frame_layout.rs
  - zircon_editor/src/ui/workbench/layout/workbench_layout.rs
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
  - zircon_editor/src/ui/workbench/layout/editor_main_frame_layout.rs
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
  - zircon_editor/src/tests/host/builtin_window_descriptors.rs
  - zircon_editor/src/tests/host/template_runtime/pane_body_documents.rs
  - zircon_editor/tests/editor_main_frame_template.rs
  - zircon_editor/tests/activity_drawer_window_template.rs
  - zircon_editor/tests/workbench_window_template.rs
  - zircon_editor/tests/asset_window_template.rs
  - zircon_editor/tests/ui_layout_editor_window_template.rs
doc_type: module-detail
---

# Activity Window Drawer Boundary

The editor main frame is now modeled separately from drawer-capable editor windows. `EditorMainFrameLayout` stores only the active window and ordered window tabs. Drawer-capable windows use `ActivityWindowLayout`, which owns `activity_drawers` and a content workspace.

The reusable `.ui.toml` shell is `editor.host.activity_drawer_window`. `WorkbenchWindow`, `AssetWindow`, and `UILayoutEditorWindow` reference that shell and mount their own left, right, bottom, and content regions. Slint remains a host projection layer; these new business window structures are `.ui.toml` assets.

`WorkbenchLayout::activity_windows()` is a transitional window-level read surface over the existing drawer storage. It lets snapshot/projection code begin consuming drawer state as window-owned state while deeper layout persistence and mutation paths are migrated incrementally.
