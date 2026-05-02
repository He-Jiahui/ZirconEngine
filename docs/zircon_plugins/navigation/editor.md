---
related_code:
  - zircon_plugins/navigation/editor/src/lib.rs
  - zircon_plugins/navigation/editor/Cargo.toml
  - zircon_plugins/navigation/runtime/src/components.rs
  - zircon_runtime/src/core/framework/navigation/mod.rs
  - zircon_runtime/src/core/framework/render/overlay.rs
  - zircon_editor/src/core/editor_extension.rs
implementation_files:
  - zircon_plugins/navigation/editor/src/lib.rs
plan_sources:
  - user: 2026-05-02 ZirconEngine navigation/pathfinding plugin completion plan
tests:
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_editor
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_runtime
doc_type: module-detail
---

# Navigation Editor Plugin

## Purpose

The navigation editor plugin exposes the authoring side of the navigation plan. It registers dedicated Navigation views, component drawers for all five runtime navigation components, bake/debug operations, and a NavMesh asset editor descriptor.

## Related Files

The plugin implementation is in `zircon_plugins/navigation/editor/src/lib.rs`. It depends on the runtime plugin for the package manifest and uses the shared navigation component ids from `zircon_runtime::core::framework::navigation`.

## Behavior Model

The editor contributes four views:

- `navigation.surfaces`
- `navigation.agents_areas`
- `navigation.bake`
- `navigation.debug_gizmos`

It also registers UI template ids for surfaces, agents/areas, bake, debug gizmos, navmesh asset inspection, and navigation settings asset inspection. Component drawers bind the editor UI documents to `NavMeshSurface`, `NavMeshModifier`, `NavMeshAgent`, `NavMeshObstacle`, and `NavMeshOffMeshLink`.

Menu/operation descriptors cover scene bake, selected surface bake, clear surface bake, settings open, gizmo toggle, navmesh asset open, and navigation settings asset open. `Navigation.Settings.Open` is wired to the existing workbench `OpenView` event for `navigation.agents_areas`, and `Navigation.Debug.ToggleGizmos` opens the navigation debug view. Bake and clear operations remain registered command descriptors until the editor host exposes a runtime-bake command handler.

## Design and Rationale

The editor never owns authoritative runtime navigation data. It contributes authoring surfaces and command descriptors, while the runtime manager owns bake/query/tick behavior. This keeps editor tooling optional and capability-gated by `editor.extension.navigation_authoring` and `editor.extension.navigation_gizmos`.

## Control Flow

`NavigationEditorPlugin::register_editor_extensions` first registers the shared drawer and four views through `editor_support`, then adds navigation-specific templates, component drawers, operations, menu items, and the asset editor descriptor. The package manifest is created by attaching the editor module to the runtime plugin manifest. The runtime framework now supplies a `NavigationGizmoSnapshot` to `SceneGizmoOverlayExtract` conversion path, giving the editor a stable bridge from baked `.znavmesh` assets to overlay line/pick-shape data.

## Edge Cases

Bake and clear operation descriptors are intentionally registered without concrete handlers in this pass, so remote invocation should not be treated as implemented bake behavior. Viewport gizmo drawing has asset-to-overlay DTO conversion for navmesh triangles and off-mesh links, but the renderer overlay pass still needs to consume and draw `SceneGizmoKind::NavigationMesh` records.

## Test Coverage

The editor plugin includes a registration test covering views, templates, component drawers, operations, and the NavMesh asset editor descriptor. A full test run is pending because the plugin workspace has concurrent long-running Cargo jobs and unrelated scaffold churn from other active plugin sessions.
