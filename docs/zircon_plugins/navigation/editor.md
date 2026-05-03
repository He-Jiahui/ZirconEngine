---
related_code:
  - zircon_plugins/navigation/editor/src/lib.rs
  - zircon_plugins/navigation/editor/Cargo.toml
  - zircon_plugins/navigation/editor/surfaces.ui.toml
  - zircon_plugins/navigation/editor/agents_areas.ui.toml
  - zircon_plugins/navigation/editor/bake.ui.toml
  - zircon_plugins/navigation/editor/debug_gizmos.ui.toml
  - zircon_plugins/navigation/editor/navmesh_asset.ui.toml
  - zircon_plugins/navigation/editor/navigation_settings_asset.ui.toml
  - zircon_plugins/navigation/editor/navmesh_surface.drawer.ui.toml
  - zircon_plugins/navigation/editor/navmesh_modifier.drawer.ui.toml
  - zircon_plugins/navigation/editor/navmesh_agent.drawer.ui.toml
  - zircon_plugins/navigation/editor/navmesh_obstacle.drawer.ui.toml
  - zircon_plugins/navigation/editor/navmesh_offmesh_link.drawer.ui.toml
  - zircon_plugins/navigation/runtime/src/components.rs
  - zircon_runtime/src/core/framework/navigation/mod.rs
  - zircon_runtime/src/core/framework/render/overlay.rs
  - zircon_editor/src/core/editor_extension.rs
implementation_files:
  - zircon_plugins/navigation/editor/src/lib.rs
  - zircon_plugins/navigation/editor/surfaces.ui.toml
  - zircon_plugins/navigation/editor/agents_areas.ui.toml
  - zircon_plugins/navigation/editor/bake.ui.toml
  - zircon_plugins/navigation/editor/debug_gizmos.ui.toml
  - zircon_plugins/navigation/editor/navmesh_asset.ui.toml
  - zircon_plugins/navigation/editor/navigation_settings_asset.ui.toml
  - zircon_plugins/navigation/editor/navmesh_surface.drawer.ui.toml
  - zircon_plugins/navigation/editor/navmesh_modifier.drawer.ui.toml
  - zircon_plugins/navigation/editor/navmesh_agent.drawer.ui.toml
  - zircon_plugins/navigation/editor/navmesh_obstacle.drawer.ui.toml
  - zircon_plugins/navigation/editor/navmesh_offmesh_link.drawer.ui.toml
plan_sources:
  - user: 2026-05-02 ZirconEngine navigation/pathfinding plugin completion plan
tests:
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_editor --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-validation --color never -vv
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-validation --message-format short --color never
doc_type: module-detail
---

# Navigation Editor Plugin

## Purpose

The navigation editor plugin exposes the authoring side of the navigation plan. It registers dedicated Navigation views, component drawers for all five runtime navigation components, bake/debug operations, NavMesh and navigation-settings asset editor descriptors, and concrete UI document descriptors for each view/drawer/asset pane.

## Related Files

The plugin implementation is in `zircon_plugins/navigation/editor/src/lib.rs`. It depends on the runtime plugin for the package manifest and uses the shared navigation component ids from `zircon_runtime::core::framework::navigation`. The authored UI documents live beside the crate root as `*.ui.toml` files so plugin packaging can resolve `plugins://navigation/editor/...` URIs without embedding editor layout strings in Rust.

## Behavior Model

The editor contributes four views:

- `navigation.surfaces`
- `navigation.agents_areas`
- `navigation.bake`
- `navigation.debug_gizmos`

It also registers UI template ids for surfaces, agents/areas, bake, debug gizmos, navmesh asset inspection, and navigation settings asset inspection. Component drawers bind the editor UI documents to `NavMeshSurface`, `NavMeshModifier`, `NavMeshAgent`, `NavMeshObstacle`, and `NavMeshOffMeshLink`.

Menu/operation descriptors cover scene bake, selected surface bake, clear surface bake, settings open, gizmo toggle, navmesh asset open, and navigation settings asset open. `Navigation.Settings.Open` is wired to the existing workbench `OpenView` event for `navigation.agents_areas`, `Navigation.Debug.ToggleGizmos` opens the navigation debug view, and the bake/clear operations open the navigation bake view. Asset open operations use the existing asset-browser event because the editor host does not yet pass a selected navigation resource payload into plugin asset panes.

## Design and Rationale

The editor never owns authoritative runtime navigation data. It contributes authoring surfaces and command descriptors, while the runtime manager owns bake/query/tick behavior. This keeps editor tooling optional and capability-gated by `editor.extension.navigation_authoring` and `editor.extension.navigation_gizmos`.

## Control Flow

`NavigationEditorPlugin::register_editor_extensions` first registers the shared drawer and four views through `editor_support`, then adds navigation-specific templates, component drawers, operations, menu items, and asset editor descriptors. The registration test asserts that every declared UI document exists on disk, which protects the `plugins://navigation/editor/...` URI surface. The package manifest is created by attaching the editor module to the runtime plugin manifest. The runtime framework now supplies a `NavigationGizmoSnapshot` to `SceneGizmoOverlayExtract` conversion path, giving the editor a stable bridge from baked `.znavmesh` assets to overlay line/pick-shape data.

## Edge Cases

Bake and clear operation descriptors open the bake view but are intentionally registered without concrete runtime bake execution in this pass, so remote invocation should not be treated as implemented bake/writeback behavior. Viewport gizmo drawing has asset-to-overlay DTO conversion for navmesh triangles and off-mesh links, but the renderer overlay pass still needs to consume and draw `SceneGizmoKind::NavigationMesh` records.

## Test Coverage

`cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_editor --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-validation --color never -vv` passed: 1 unit test and doctests. The first non-verbose attempt failed before reaching this crate with transient MSVC `LNK1181` while linking a third-party proc-macro dependency; the verbose rerun completed successfully and validated navigation editor registration plus UI document existence.
