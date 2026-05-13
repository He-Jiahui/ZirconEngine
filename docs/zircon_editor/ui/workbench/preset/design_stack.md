---
related_code:
  - zircon_editor/src/ui/host/builtin_layout/builtin_shell_view_instances.rs
  - zircon_editor/src/ui/host/builtin_layout/hybrid_layout.rs
  - zircon_editor/src/ui/host/builtin_layout/mod.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/functional_panel_view_descriptors.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/functional_window_view_descriptors.rs
  - zircon_editor/src/ui/host/builtin_views/builtin_view_descriptors.rs
  - zircon_editor/src/ui/workbench/preset/mod.rs
  - zircon_editor/src/ui/workbench/preset/default_layout.rs
  - zircon_editor/src/ui/workbench/preset/default_registry.rs
  - zircon_editor/src/ui/workbench/preset/design_stack.rs
  - zircon_editor/src/ui/workbench/preset/functional_window.rs
  - zircon_editor/src/ui/workbench/preset/panel_preset.rs
  - zircon_editor/src/ui/workbench/preset/shell_preset.rs
  - zircon_editor/src/ui/workbench/mod.rs
  - zircon_runtime/src/ui/component/catalog/material_foundation.rs
  - zircon_runtime_interface/src/ui/skin/preset.rs
implementation_files:
  - zircon_editor/src/ui/host/builtin_layout/builtin_shell_view_instances.rs
  - zircon_editor/src/ui/host/builtin_layout/hybrid_layout.rs
  - zircon_editor/src/ui/host/builtin_layout/mod.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/functional_panel_view_descriptors.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/functional_window_view_descriptors.rs
  - zircon_editor/src/ui/host/builtin_views/builtin_view_descriptors.rs
  - zircon_editor/src/ui/workbench/preset/mod.rs
  - zircon_editor/src/ui/workbench/preset/default_layout.rs
  - zircon_editor/src/ui/workbench/preset/default_registry.rs
  - zircon_editor/src/ui/workbench/preset/design_stack.rs
  - zircon_editor/src/ui/workbench/preset/functional_window.rs
  - zircon_editor/src/ui/workbench/preset/panel_preset.rs
  - zircon_editor/src/ui/workbench/preset/shell_preset.rs
plan_sources:
  - user: 2026-05-11 Implement Material + Fyrox + JetBrains + Unreal editor UI plan
  - .codex/plans/Zircon Editor UI Material  Fyrox  JetBrains  Unreal.md
tests:
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup.rs
  - zircon_editor/src/tests/host/builtin_window_descriptors.rs
  - zircon_editor/src/ui/workbench/preset/default_layout.rs
  - zircon_editor/src/ui/workbench/preset/default_registry.rs
  - zircon_editor/src/ui/workbench/preset/design_stack.rs
  - cargo check -p zircon_editor --lib --locked --target-dir target/codex-editor-ui-shell (2026-05-11: passed)
  - cargo test -p zircon_editor --lib default_stack --locked --target-dir target/codex-editor-ui-shell (2026-05-11: passed, 8 passed)
  - cargo test -p zircon_editor --lib default_layout --locked --target-dir target/codex-editor-ui-shell (2026-05-11: passed, 5 passed)
  - cargo test -p zircon_editor --lib default_registry --locked --target-dir target/codex-editor-ui-shell (2026-05-11: passed, 2 passed)
  - cargo test -p zircon_editor --lib applying_project_workspace_preserves_builtin_shell_drawers --locked --target-dir target/codex-shared-b (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_editor --lib bootstrap_and_startup --locked --target-dir target/codex-shared-b (2026-05-11: passed, 10 passed)
  - cargo test -p zircon_editor --lib builtin_activity_windows --locked --target-dir target/codex-shared-b (2026-05-11: passed, 2 passed after rerun with longer timeout)
  - cargo test -p zircon_editor --lib unreal_style_feature_window_descriptors_use_expected_hosts --locked --target-dir target/codex-shared-b (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_editor --lib default_design_stack --locked --target-dir target/codex-shared-b (2026-05-11: passed, 2 passed)
  - cargo test -p zircon_editor --lib functional_editor_internal_view_descriptors_use_document_host --locked --target-dir target/codex-shared-b (2026-05-11: passed, 1 passed)
doc_type: module-detail
---

# Editor UI Design Stack

`EditorUiDesignStack::material_fyrox_jetbrains_unreal()` is the editor-side product preset for the new UI direction. It binds the shared interface preset IDs to concrete editor window roles without taking over layout mutation, rendering, or runtime scene state.

## Preset Split

The stack records four independent preset IDs:

- `material_dark` for visual tokens and primitive component states;
- `fyrox_panel` for the content and interaction model of editor panels;
- `jetbrains_shell` for side drawers, tabs, and floating-window shell behavior;
- `unreal_window_model` for functional editor windows as top-level editing units.

This makes later Unreal or JetBrains style convergence a preset change instead of a rewrite of panel data models.

## Fyrox Panel Contracts

The stack owns a `panels: Vec<FyroxPanelPreset>` collection keyed by view id. A panel preset is not a renderer and does not bind the panel to one `.ui.toml` asset. It records the logical component roles and expected interactions that make a Fyrox-like editor panel:

- Hierarchy uses SearchField, TreeView, and ContextMenu roles with search/filter, selection sync, and context menu interaction.
- Inspector uses PropertyGrid, InspectorSection, and FieldEditor roles with property edit and selection sync interaction.
- Asset Browser uses FolderTree, AssetGrid, AssetList, PreviewPane, and MetadataPane roles with search/filter, preview, metadata edit, and context menu interaction.
- Scene/Game and Prefab viewport views use ViewportHost, PaneToolbar, and GizmoControls roles where appropriate.
- Console and Runtime Diagnostics use FilterBar, VirtualList, SeverityChips/PropertyGrid roles with severity filtering, virtualization, and refresh interaction.
- Plugin Manager uses SearchField, CategorizedList, and StatusActionControls with plugin enable/disable interaction.

Every primary view and drawer view declared by the default functional windows must have a matching `FyroxPanelPreset`. That keeps panel composition explicit before the retained host renders it and gives the future Unreal/JetBrains convergence path a stable domain contract to restyle or remap.

Each `FyroxPanelComponentRole` also maps to a Material foundation component id through `component_id()`. The default stack validates that every role used by every panel exists in `UiComponentDescriptorRegistry::material_editor_foundation()`, so a new panel role cannot remain an abstract enum with no component contract behind it.

## JetBrains Shell Contract

The stack owns a `shell: JetBrainsShellPreset` value that describes the workbench shell behavior independently from the current concrete `WorkbenchLayout` projection. It groups drawer tabs by canonical slot and gives each drawer a product label, default mode, visible view ids, detach/attach permissions, activity-bar collapse behavior, and persistence expectations.

The default shell mirrors the first target layout:

- `LeftTop` is the Project Tools drawer and exposes Hierarchy plus Asset Browser;
- `LeftBottom` is the Modules drawer and starts collapsed with Plugin Manager;
- `RightTop` is the Inspector drawer;
- `Bottom` is the Output drawer and exposes Console, Runtime Diagnostics, and Desktop Export.

`JetBrainsTabBehavior` records the tab-shell rules that later retained-host work should honor: tab reordering, activation on drop, middle-click close for document tabs, and guarded close behavior for tool tabs. `JetBrainsFloatingWindowBehavior` records the detach model: views detach to native windows, can attach back to their source drawer, take focus on detach, persist floating geometry, and restore hidden drawers when they return.

`default_stack_shell_contract_covers_workbench_drawer_views` guards the first integration boundary. Any drawer view declared by the main Workbench window must appear in the JetBrains shell contract before the default layout can claim to expose it. This keeps the future Unreal and JetBrains styling passes tied to the same drawer model.

`default_workbench_layout()` now consults `JetBrainsShellPreset` when routing drawer views and when choosing drawer startup mode. Workbench drawer placement therefore comes from the preset: Project Tools and Output start pinned, while the Modules drawer starts collapsed even though it owns the Plugin Manager tab. Functional editor windows also reuse shell mappings for shared views such as Inspector and fall back to the conservative view-id classifier only for window-local views that are not part of the global Workbench shell contract.

## Unreal Window Model

The stack owns a `window_model: UnrealWindowModelPreset` value that declares eight functional windows:

- the main workbench window with Scene/Game documents and hierarchy/inspector/asset browser/console/diagnostics/build export/plugin drawers;
- a Scene/Game document window;
- Prefab, Material, UI Asset, and Animation editor windows with floating allowed;
- Asset Browser and Diagnostics windows that are drawer-backed.

This mirrors Unreal's model where a feature editor is a window/workbench, while still allowing JetBrains-like drawer tabs and Fyrox-style panel content inside each window. `UnrealWindowModelPreset` exposes derived queries for the main Workbench, floating feature editors, and drawer-backed utility windows, so later Unreal convergence can change window policy without rewriting the panel and shell presets.

## Default Layout Projection

`EditorUiDesignStack::default_workbench_layout()` projects the preset into the existing neutral `WorkbenchLayout` model. The default projection places Scene/Game in the central document tab stack, maps Fyrox panel views into JetBrains-style drawers, and registers every functional editor window as an `ActivityWindowLayout`.

Default drawer routing is intentionally data-driven and conservative. Workbench views are routed through `JetBrainsShellPreset.visible_views`; window-local feature-editor drawer views use the same shell mapping when a view id is shared with the Workbench, then fall back to the legacy classifier:

- hierarchy and asset browser go to the left tool drawer;
- inspector and metadata go to the right tool drawer;
- console, diagnostics, and build export go to the bottom drawer;
- plugin manager goes to the lower-left tool drawer.

Floating-allowed feature editors such as Material, Prefab, UI Asset, and Animation are registered with native-window host mode, but the projection does not force all of them open on startup. Open/close, detach/attach, and focus still flow through `LayoutManager` and `LayoutCommand`.

The generated `ActivityWindowLayout.descriptor_id` values are also covered by builtin view descriptors. `functional_window_view_descriptors.rs` registers the preset-aligned `_window` ids for Scene/Game, Prefab, Material, UI Asset, Animation, Asset Browser, and Diagnostics, while older descriptor ids remain available for legacy menu and asset-editor paths. `functional_panel_view_descriptors.rs` covers the internal functional-editor page views such as Material Graph, UI Designer, Animation Timeline, Asset Preview, and Asset Metadata, so `default_view_instances()` never creates a descriptor id that the host registry cannot recognize.

## Default Registry Projection

`EditorUiDesignStack::default_view_instances()` creates stable `ViewInstance` metadata for every preset primary view and drawer view. Workbench views keep the historical `#1` instance suffix, while functional editor windows receive window-scoped suffixes such as `#material_editor`. This prevents the Material Editor inspector from colliding with the Workbench inspector once both are present in the window registry.

`EditorUiDesignStack::default_window_registry()` then syncs the preset layout plus those view instances through `EditorWindowRegistry::sync_from_layout(...)`. This gives the new preset a single query surface for active window, drawer ownership, selected drawer, dock position, and native-window host mode.

## Startup Default Layout

`builtin_hybrid_layout_for_subsystems(...)` now delegates to `EditorUiDesignStack::material_fyrox_jetbrains_unreal().default_workbench_layout()` and only applies capability filtering afterward. The default editor startup therefore no longer begins from the legacy Project-first drawer layout. It starts with:

- central Scene/Game document tabs;
- left drawer tabs for Hierarchy and Asset Browser, with Hierarchy selected;
- right drawer tab for Inspector;
- bottom drawer tabs for Console, Runtime Diagnostics, and Build Export;
- lower-left drawer tab for Module Plugins.

When the runtime diagnostics subsystem is unavailable, `builtin_hybrid_layout_for_subsystems(...)` removes `editor.runtime_diagnostics#1` from both the root drawer map and the workbench activity-window drawer map, then collapses any drawer that becomes empty. The shell view instance seed follows the same preset direction by dropping the legacy `editor.project#1` seed and retitling `editor.assets#1` as `Asset Browser`.

The previous `layout_drawers.rs` and `workbench_page.rs` sources remain on disk as legacy references, but `builtin_layout::mod` no longer wires them into the startup path. They should not be used as the design source for new workbench behavior unless the UI plan explicitly calls for a legacy comparison.

## Boundaries

The preset is a data model only. It does not alter `WorkbenchLayout`, does not create native windows, and does not move editor/runtime authority. Window placement still flows through the existing layout manager, floating-window layout, and retained host projection paths.
