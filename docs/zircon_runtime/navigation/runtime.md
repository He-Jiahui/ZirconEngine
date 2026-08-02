---
related_code:
  - zircon_runtime/src/navigation/mod.rs
  - zircon_runtime/src/navigation/module.rs
  - zircon_runtime/src/navigation/operation/mod.rs
  - zircon_runtime/src/navigation/operation/handler.rs
  - zircon_runtime/src/navigation/operation/registration.rs
  - zircon_runtime/src/navigation/repath_budget.rs
  - zircon_runtime/src/navigation/runtime.rs
  - zircon_runtime/src/navigation/runtime/avoidance.rs
  - zircon_runtime/src/navigation/runtime/baked_mesh.rs
  - zircon_runtime/src/navigation/runtime/baked_mesh/query_scratch.rs
  - zircon_runtime/src/navigation/runtime/baked_mesh/spatial_index.rs
  - zircon_runtime/src/navigation/runtime/math.rs
  - zircon_runtime/src/navigation/runtime/state.rs
  - zircon_runtime/src/navigation/runtime/tests.rs
  - zircon_runtime/src/navigation/runtime/world_scan.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host/navigation.rs
  - examples/vampire/assets/navigation/main.navmesh.toml
implementation_files:
  - zircon_runtime/src/navigation/mod.rs
  - zircon_runtime/src/navigation/module.rs
  - zircon_runtime/src/navigation/operation/mod.rs
  - zircon_runtime/src/navigation/operation/handler.rs
  - zircon_runtime/src/navigation/operation/registration.rs
  - zircon_runtime/src/navigation/repath_budget.rs
  - zircon_runtime/src/navigation/runtime.rs
  - zircon_runtime/src/navigation/runtime/avoidance.rs
  - zircon_runtime/src/navigation/runtime/baked_mesh.rs
  - zircon_runtime/src/navigation/runtime/baked_mesh/query_scratch.rs
  - zircon_runtime/src/navigation/runtime/baked_mesh/spatial_index.rs
  - zircon_runtime/src/navigation/runtime/math.rs
  - zircon_runtime/src/navigation/runtime/state.rs
  - zircon_runtime/src/navigation/runtime/tests.rs
  - zircon_runtime/src/navigation/runtime/world_scan.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
plan_sources:
  - user: 2026-06-11 vampire roguelite runtime example and screenshot validation
  - docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/engine-code-structure-convention.md
  - dev/godot/modules/navigation_2d
  - dev/godot/modules/navigation_3d
tests:
  - python -m unittest tools.tests.test_runtime_module_family_boundary
  - cargo check -p zircon_runtime --lib --message-format short --color never
  - zircon_runtime/src/navigation/module.rs::tests::builtin_navigation_module_obeys_driver_manager_dependency_layers
  - cargo test -p zircon_runtime --lib builtin_host_modules_register_gameplay_capabilities --message-format short --color never
  - cargo test -p zircon_runtime --lib vampire_example_manifest_scene_and_scripts_are_importable --message-format short --color never
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs::runtime_navigation_boundary_file_set_requires_doc_update
  - zircon_runtime/src/navigation/runtime/tests.rs::tick_world_agent_moves_only_selected_agent_and_avoids_local_colliders
  - zircon_runtime/src/navigation/runtime/tests.rs::navigation_manager_accessors_recover_poisoned_state_lock
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs::runtime_15_navigation_lock_poison_recovery_guard_covers_builtin_navigation_manager
  - "pending: cargo test -p zircon_runtime --lib navigation --locked"
doc_type: module-detail
---

# Runtime Navigation Module

## Purpose

`zircon_runtime::navigation` is the built-in lightweight navigation module used by standalone runtime sessions. It loads project-authored baked navmesh assets and exposes enough runtime pathing for project gameplay to move agents through the scene without requiring the external navigation plugin workspace to be linked into the executable.

The module registers with the canonical `navigation.runtime` identity and provides the runtime navigation service consumed by gameplay host functions.

## Project Loading

`dynamic_api::session::project` looks for `assets/navigation/main.navmesh.toml` under the selected project directory. When present, the session loads the baked graph into the navigation runtime before project update callbacks start moving enemies.

The vampire example uses this path for the clearing scene. Runtime obstacles are then installed by gameplay startup for authored props so enemy movement respects scene blockers.

## Runtime Behavior

The runtime stores baked vertices, polygons, and graph edges, then supports:

- nearest-position sampling on the baked surface
- pathfinding across the baked graph
- simple ray/visibility checks for straight-line traversal
- agent ticking toward dynamic destinations
- local separation from obstacles and nearby agents

The folder-backed runtime owner split keeps those responsibilities in focused files without changing the public `BuiltinNavigationManager` contract:

- `runtime.rs` owns manager construction, service trait orchestration, mesh selection, public method routing, and per-agent tick flow.
- `runtime/state.rs` owns loaded mesh handles, selected mesh state, runtime settings, and stats.
- `runtime/baked_mesh.rs` owns baked polygon orchestration and path result helpers; `runtime/baked_mesh/spatial_index.rs` owns the spatial candidate index and `runtime/baked_mesh/query_scratch.rs` owns reusable query scratch state.
- `runtime/world_scan.rs` owns ECS component scanning for runtime agents, nearby agents, and runtime obstacles.
- `runtime/avoidance.rs` owns local obstacle and agent separation.
- `runtime/math.rs` owns shared navigation math helpers.
- `runtime/tests.rs` owns focused fallback runtime behavior tests.

The separate folder-backed operation integration exposes runtime-authoritative navigation authoring work through the shared operation service without moving editor state into the runtime module:

- `operation/mod.rs` is the narrow feature-domain entry and exports only handler registration.
- `operation/registration.rs` registers bake-scene, bake-surface, clear-surface, and restore-snapshot operation ids with `RuntimeOperationService`.
- `operation/handler.rs` validates typed JSON payloads, resolves the scene navigation driver, applies bake/clear/restore work against the runtime world, and returns before/after snapshots for transaction undo/redo.

These operation files are runtime integration owners, not a second editor command stack. Editor factories and retained UI routing remain in `zircon_editor` and `zircon_plugins/navigation/editor`.

Gameplay calls `nav_move_towards_entity(...)`, which writes the destination agent component to the mover and invokes the navigation runtime. This keeps enemy chase behavior tied to the baked scene instead of raw per-frame translation.

## Service Registration Layers

The built-in module registers one implementation owner and two public facades without reversing the Runtime service hierarchy:

- `navigation.runtime.Driver.BuiltinNavigationRuntime` owns the shared `BuiltinNavigationManager` implementation as an internal Driver with no dependencies.
- `navigation.runtime.Driver.SceneNavigationRuntime` is the scene-facing Driver facade and depends only on the internal Driver.
- `navigation.runtime.Manager.NavigationManager` is the public query Manager facade and depends downward on the same internal Driver.

Both facades resolve the internal implementation with `resolve_driver(...)`, so they share one cached runtime instance. A Driver never depends on a Manager. The retired `navigation.runtime.Manager.BuiltinNavigationManager` identity is not retained as an alias, re-export, or compatibility registration.

`builtin_navigation_module_obeys_driver_manager_dependency_layers` locks the descriptor kinds and dependency names, registers the complete descriptor through `CoreRuntime`, and resolves the internal Driver, scene Driver, and public Manager. Product acceptance additionally builds `zircon_app --bin zircon_editor` and starts both the default Editor and `editor.runtime_diagnostics` views with isolated configuration.

## Poison Handling

`BuiltinNavigationManager` stores runtime navigation state behind one mutex. Runtime 15 M3 navigation lock poison recovery moved every production state access through the private `lock_state()` helper, which recovers poisoned locks with `unwrap_or_else(|poisoned| poisoned.into_inner())` instead of panicking on `expect("navigation state lock poisoned")`.

Status: `runtime_15_navigation_lock_poison_recovery_static_passed_cargo_deferred`.

The recovery policy covers navmesh loading, navigation settings loading, path/sample/raycast queries, agent tick stats, `tick_agent(...)` path/sample lookups, and `stats()`. It does not change `NavigationManager`, baked navmesh serialization, selected-mesh semantics, or the boundary between this built-in fallback and the external Recast-backed plugin stack.

`navigation_manager_accessors_recover_poisoned_state_lock` deliberately poisons the state lock in test-only code and then proves load/settings/sample/stats still work. `runtime_15_navigation_lock_poison_recovery_guard_covers_builtin_navigation_manager` keeps the helper, test, docs, and status anchors synchronized so the production manager cannot silently return to direct lock unwrap or lock-poison panic behavior.

## Scope

This module is a runtime-owned bridge for project examples and standalone gameplay validation. The fuller Recast-backed plugin stack under `zircon_plugins/navigation` remains the long-term owner for advanced baking, Detour/TileCache integration, editor authoring, and plugin catalog capability reporting.

The current runtime navigation module does not replace that plugin. It gives `zircon_runtime` an independent, minimal, testable pathfinding capability so examples such as `examples/vampire` can run without requiring plugin-workspace runtime composition.

## Runtime 14 Boundary Judgment

Runtime 14 corrected the earlier "thin navigation module" assumption. The module now has 15 Rust owner files after the folder-backed runtime, operation integration, repath-budget, and baked-mesh query splits. `runtime.rs` remains the manager owner for real fallback behavior: baked navmesh loading, nearest-position sampling, pathfinding, raycast checks, world-agent ticking, obstacle collection, and local avoidance. `operation/{mod,handler,registration}.rs` adds the narrow shared-operation integration; `repath_budget.rs` bounds repath admission, while `runtime/baked_mesh/{query_scratch,spatial_index}.rs` keep query state and spatial indexing out of the orchestration owner. None moves editor authoring state into the root wiring files.

The crate-root seat remains intentional. `core::framework::navigation` owns contracts and DTOs, while `zircon_runtime::navigation` owns the built-in fallback implementation used when the external Recast-backed plugin stack is not linked into the process.

Godot remains a useful layering reference, but the verified source paths are `dev/godot/modules/navigation_2d` and `dev/godot/modules/navigation_3d`, not a single `modules/navigation` directory. Zircon should also avoid adopting "server" terminology here because this module is not a network service.

The Runtime 14 judgement is:

- Keep `navigation` at crate root as a self-contained runtime fallback.
- Do not split behavior into more root-level navigation families unless baking/editor/Recast ownership moves into `zircon_runtime`, which is currently a non-goal.
- Keep the folder-backed runtime owner split documented whenever `runtime/avoidance.rs`, `runtime/baked_mesh.rs`, `runtime/world_scan.rs`, `runtime/state.rs`, `runtime/math.rs`, or `runtime/tests.rs` changes ownership.
- Keep the operation integration folder-backed and document any change to `operation/mod.rs`, `operation/handler.rs`, or `operation/registration.rs`; do not fold handler behavior into `module.rs` or `runtime.rs`.
- If future code adds new navigation behavior files, document whether they extend the fallback runtime or belong in `zircon_plugins/navigation`.
`runtime_navigation_boundary_file_set_requires_doc_update` asserts the current file set and forces new behavior files to update this document before expanding the fallback runtime.
