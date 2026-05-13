---
related_code:
  - zircon_editor/src/ui/workbench/preset/design_stack.rs
  - zircon_editor/src/ui/workbench/preset/panel_preset.rs
  - zircon_runtime/src/ui/component/catalog/material_foundation.rs
  - zircon_runtime/src/ui/component/catalog/mod.rs
  - zircon_runtime/src/ui/tests/component_catalog.rs
  - zircon_runtime/src/ui/tests/v2_asset.rs
  - zircon_editor/assets/ui/editor/fyrox_panel_demo_window.v2.ui.toml
  - zircon_editor/assets/ui/editor/layout_demo_window.v2.ui.toml
  - zircon_runtime_interface/src/ui/skin/preset.rs
implementation_files:
  - zircon_runtime/src/ui/component/catalog/material_foundation.rs
  - zircon_runtime/src/ui/component/catalog/mod.rs
  - zircon_editor/assets/ui/editor/fyrox_panel_demo_window.v2.ui.toml
  - zircon_editor/assets/ui/editor/layout_demo_window.v2.ui.toml
plan_sources:
  - user: 2026-05-11 Implement Material + Fyrox + JetBrains + Unreal editor UI plan
  - .codex/plans/Zircon Editor UI Material  Fyrox  JetBrains  Unreal.md
  - .codex/plans/ZirconEngine UI 组件化与 Material 样式器重构计划.md
tests:
  - zircon_runtime/src/ui/tests/component_catalog.rs
  - zircon_editor/src/ui/workbench/preset/design_stack.rs
  - cargo check -p zircon_runtime --lib --locked --target-dir target/codex-editor-ui-shell (2026-05-11: passed)
  - cargo test -p zircon_runtime --lib material_editor_foundation_catalog_covers_planned_component_layers --locked --target-dir target/codex-editor-ui-shell (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_runtime --locked --verbose (2026-05-12 runtime closeout: passed after `TreeView.expanded` schema coverage was restored; runtime lib target reported 1268 passed, 0 failed)
  - cargo test -p zircon_runtime --lib layout_demo_window_compiles_with_window_drawer_and_data_view_components --locked --target-dir target/codex-editor-ui-shell (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_runtime --lib fyrox_panel_demo_window_compiles_with_all_panel_role_components --locked --target-dir target/codex-editor-ui-shell (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_editor --lib default_stack --locked --target-dir target/codex-editor-ui-shell (2026-05-11: passed, 8 passed)
doc_type: module-detail
---

# Material Editor Foundation Catalog

`UiComponentDescriptorRegistry::material_editor_foundation()` is the first component catalog for the new editor UI direction. It is intentionally separate from `editor_showcase()` so existing showcase tests and legacy showcase consumers keep their exact component set while the new workbench path gains a clean Material-first contract.

## Component Layers

The catalog is organized around the implementation plan's four layers:

- primitive controls: button, icon button, text field, checkbox, switch, dropdown, slider, tabs, menu, tooltip, scrollbar, splitter, panel, modal;
- layout controls: flex group, horizontal group, vertical group, grid group, overlay, scroll view;
- data/view controls: list view, virtual list, tree view, property grid, inspector section;
- Fyrox panel role controls: search field, context menu, field editor, folder tree, asset grid/list, preview and metadata panes, viewport host, pane toolbar, gizmo controls, filter bar, severity chips, categorized list, status action controls, graph canvas, source editor, timeline, and visual designer;
- shell controls: window, view, drawer, view tab, window frame, document node, tab stack, floating window, dock host, workbench shell, slot, and composite.

Every descriptor receives the `material_dark`, `material`, and `material-dark` default classes. Each descriptor also declares the same visual state schema used by the Material skin: hovered, pressed, selected, disabled, focused, error, and warning.

## Behavior Contracts

The foundation descriptors now carry the first behavior-level props and events needed by the planned Workbench shell:

- `VirtualList` declares `item_count`, `item_extent`, `overscan`, and `SetVisibleRange`.
- `TreeView` declares `query` and `expanded` props plus selection, expand/collapse, and context-menu events.
- `PropertyGrid` and `InspectorSection` keep inspector-style section/field slots and property-edit events.
- `SearchField`, `FilterBar`, and `SeverityChips` declare query/severity props and selection/value-change events for Hierarchy, Asset Browser, Console, and Diagnostics.
- `FolderTree`, `AssetGrid`, `AssetList`, and `CategorizedList` declare collection slots plus selection, expand/collapse, context-menu, locate, and open-reference events.
- `PreviewPane`, `MetadataPane`, `FieldEditor`, and `StatusActionControls` give Inspector, Asset Browser, Plugin Manager, and export views reusable editor-only composite surfaces.
- `ViewportHost`, `GraphCanvas`, and `VisualDesigner` are canvas-layout components for Scene/Game, Material, UI, and animation authoring surfaces.
- `PaneToolbar`, `GizmoControls`, `SourceEditor`, and `Timeline` cover viewport/editor toolbars, transform controls, text-source editing, and animation scrubbing.
- `DocumentNode`, `TabStack`, and `FloatingWindow` expose the Workbench split/tab/detach model as authorable shell components.
- `Drawer`, `View`, `ViewTab`, `Window`, `WindowFrame`, `DockHost`, and `WorkbenchShell` declare stable ids, active-selection/focus props, preset ids, and shell interaction events.

These are still neutral component descriptors. They do not create native windows or bind editor state directly; they let v2 assets and editor presenters agree on which values and events are valid before a concrete skin or host renders them.

`FyroxPanelComponentRole::component_id()` maps editor panel roles onto these descriptor ids. The editor-side default stack test walks every panel preset role and verifies that the Material foundation catalog contains the matching descriptor. Adding a new Fyrox panel role now requires either reusing an existing component id or adding a catalog descriptor with matching host/render capabilities.

## Host Capabilities

Editor shell descriptors require `UiHostCapability::Editor`, which keeps them out of runtime-basic host palettes. `VirtualList` requires both `UiHostCapability::VirtualizedLayout` and `UiRenderCapability::VirtualizedLayout`, preserving the existing validation invariant that virtualized rendering must be backed by a virtualized host.

Editor panel controls also require `UiHostCapability::Editor`. Text-backed controls such as `SearchField` and `SourceEditor` require `TextInput`; canvas-backed controls such as `ViewportHost`, `GraphCanvas`, and `VisualDesigner` require `CanvasRender`; preview panes require `ImageRender`.

## Layout Roles

The catalog uses `UiComponentLayoutRole` rather than renderer-specific type checks:

- flex, horizontal, vertical, and scroll views remain flex-family layouts for the Taffy bridge;
- grid group maps to grid;
- overlay stays Zircon-owned;
- virtual list stays explicitly virtualized;
- viewport, graph, and designer surfaces use the canvas layout role;
- dock host uses the editor dock role for future JetBrains/Unreal workbench shell assembly.
