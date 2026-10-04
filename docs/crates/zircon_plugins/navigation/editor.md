---
related_code:
  - zircon_plugins/navigation/editor/src/plugin.rs
  - zircon_plugins/navigation/editor/src/plugin/registration/mod.rs
  - zircon_plugins/navigation/editor/src/bake_panel.rs
  - zircon_plugins/navigation/editor/src/overlay.rs
  - zircon_plugins/navigation/editor/src/runtime_mirror.rs
  - zircon_plugins/navigation/editor/src/viewport_overlay_provider.rs
  - zircon_plugins/navigation/runtime/src/overlay_frame.rs
  - zircon_plugins/navigation/runtime/src/agent.rs
  - zircon_runtime/src/core/framework/navigation/agent.rs
  - zircon_plugins/navigation/editor/bake.zui
  - zircon_plugins/navigation/editor/debug_gizmos.zui
  - zircon_runtime/src/core/framework/navigation/gizmo.rs
  - zircon_runtime/src/core/framework/render/overlay.rs
implementation_files:
  - zircon_plugins/navigation/editor/src/plugin.rs
  - zircon_plugins/navigation/editor/src/plugin/registration/assets.rs
  - zircon_plugins/navigation/editor/src/plugin/registration/components.rs
  - zircon_plugins/navigation/editor/src/plugin/registration/operations.rs
  - zircon_plugins/navigation/editor/src/plugin/registration/templates.rs
  - zircon_plugins/navigation/editor/src/bake_panel.rs
  - zircon_plugins/navigation/editor/src/overlay.rs
  - zircon_plugins/navigation/editor/src/runtime_mirror.rs
  - zircon_plugins/navigation/editor/src/viewport_overlay_provider.rs
  - zircon_plugins/navigation/editor/bake.zui
  - zircon_plugins/navigation/editor/debug_gizmos.zui
plan_sources:
  - docs/plans/zircon_plugins/05-navigation.md
  - docs/plans/zircon_plugins/10-editor-integration.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_plugins/navigation/editor/src/tests.rs
doc_type: module-detail
---

# Navigation Editor Plugin

## Purpose

The Navigation editor package owns optional authoring and diagnostics UI. Runtime navigation state remains authoritative in `zircon_plugin_navigation_runtime`; the editor submits typed bake intents, consumes progress/debug mirror frames, and projects those frames into retained `.zui` views and the shared scene-gizmo overlay contract.

## PIE event consumer

The plugin declares `navigation.editor.overlay_frame` as the typed consumer of `navigation.events.overlay_frame` with schema `navigation.events.overlay_frame.v1`. The declaration is projected into `navigation.editor` in the package manifest and retains the same `NavigationPieMirror` instance used by the viewport provider.

The editor host subscribes only while PIE is active and `editor.extension.navigation_gizmos` is enabled. Cross-session deliveries, wrong schemas, foreign handles, stale sequences, and stale NavMesh owner generations are rejected before the mirror changes. Exiting PIE unsubscribes and clears the complete overlay frame.

## Module Boundaries

- `plugin.rs` owns the explicit editor plugin declaration and the single shared `NavigationPieMirror`; the runtime consumer and provider registration both receive that same `Arc`.
- `plugin/registration/` owns extension registration beneath the plugin declaration owner, split into assets, component drawers, commands/menu items, templates, and the viewport provider.
- `bake_panel.rs` owns the editor-side bake request and progress state machine. Its controller submits only through `NavigationBakeBackend`, maps bake actions to framework `NavMeshBakeRequest`, and does not hold a `World` or call `NavigationManager` directly.
- `overlay.rs` owns neutral conversion from `NavigationGizmoSnapshot` plus a PIE frame into `SceneGizmoOverlayExtract`.
- `viewport_overlay_provider.rs` owns the real `navigation.viewport.overlay.provider` factory and reads only the shared mirror.
- `runtime_mirror.rs` owns the read-only PIE frame cache. It consumes the runtime-owned `NavigationOverlayFrame`, which contains canonical NavMesh geometry plus the current agent report; frames are accepted only for the active play session, increasing delivery sequence, and non-decreasing owner generation.

This split keeps the root/entry files thin and prevents registration, retained layout, runtime mirroring, and overlay projection from accumulating in one file.

## Bake Panel

`NavigationBakePanel` accepts scene bake, selected-surface bake, and selected-surface clear intents. One request may be active at a time. Bake/Clear descriptors stay pending edit operations with payload schemas; they are not overwritten by OpenView events. Bake actions preserve `force_full_rebuild` in the framework request. Progress phase and fraction are monotonic. Completion stores the latest `NavMeshBakeReport`; failure clears stale reports and stores a diagnostic string.

`bake.zui` follows the `ai-navmesh-ai-layout.png` structure at plugin-view scale:

- left: NavMesh surface selection;
- center: bake settings and commands;
- right: output and diagnostics;
- bottom: progress and state text.

The three command routes are `navigation.bake.scene`, `navigation.bake.surface`, and `navigation.bake.clear_surface`.

## Viewport Overlay

The overlay command and menu entry use the required path `View/Debug Overlays/Navigation`. It remains a non-undoable view-state operation with a payload schema and names the `navigation.viewport.overlay.provider` provider under the gizmo capability. The plugin registers that provider with the shared Editor05 host registry. Capability disable and PIE shutdown remove its next extract without a Navigation-specific viewport mutation path.

Area-colored NavMesh triangle/link geometry comes from the framework-owned `NavigationGizmoSnapshot`. Editor projection can independently enable NavMesh areas, off-mesh links, agent paths, desired velocity, and avoidance velocity. Agent pick spheres and path/vector lines are appended to the same `SceneGizmoKind::NavigationMesh` extract, so renderer ownership stays in the shared overlay pipeline.

## PIE Read-only Mirror

When `NavigationDebugCapture` is enabled, the runtime overlay frame carries canonical NavMesh triangles/links and serializable per-agent position, destination, queried path, desired velocity, and avoidance velocity; capture is disabled by default. `NavigationPieMirror::apply_overlay_frame` adds editor-owned play-session and delivery-sequence metadata without duplicating either geometry or the agent vector. Cross-session frames, non-increasing sequences, and stale owner generations are rejected.

The mirror exposes getters only; it has no runtime world reference and no reflect-write API. `debug_gizmos.zui` labels this boundary explicitly and presents overlay toggles, the shared overlay viewport, mirror status, and the agent list.

## Registration and Capability Behavior

The plugin registers four views, the shared navigation drawer, five component drawers, NavMesh/settings asset editors, templates, command/menu descriptors, and one real viewport overlay provider. Bake authoring commands require `editor.extension.navigation_authoring`; the overlay command, menu item, and provider require `editor.extension.navigation_gizmos`.

All extension registration goes through `EditorExtensionRegistry`; there is no editor-internal side channel.

## Tests

The editor contract suite covers:

- complete extension registration and document existence;
- bake backend submission, framework request/force-full mapping, command schema/undo metadata, rejection, monotonic progress, and completion;
- stable overlay mode/menu/capability/provider registration and live sink toggle/submission;
- distinct multi-area colors plus agent path/avoidance projection;
- runtime-produced serializable debug payload and PIE typed-event session/sequence rejection;
- SDK declaration/manifest parity and structured missing-capability diagnostics;
- required retained-layout controls for bake, clear, progress, overlay toggles, and read-only agent mirroring.

The pre-review package version completed build and 6 unit tests. The expanded corrected suite is not accepted yet: Editor03 operation factory, Editor05 provider host, and Plugins12 mirror consumer handoffs remain open, while an unrelated Render11 lightmap migration currently prevents current-source `zircon_runtime` compilation. Validation uses `zircon_plugins/Cargo.toml` and the external target `E:\cargo-targets\zircon-navigation-m6-editor`.
