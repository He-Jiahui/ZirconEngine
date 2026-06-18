---
related_code:
  - zircon_runtime/src/navigation/mod.rs
  - zircon_runtime/src/navigation/module.rs
  - zircon_runtime/src/navigation/runtime.rs
  - zircon_runtime/src/navigation/runtime/avoidance.rs
  - zircon_runtime/src/navigation/runtime/baked_mesh.rs
  - zircon_runtime/src/navigation/runtime/math.rs
  - zircon_runtime/src/navigation/runtime/state.rs
  - zircon_runtime/src/navigation/runtime/world_scan.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/vampire_gameplay/world_ops.rs
  - examples/vampire/assets/navigation/main.navmesh.toml
implementation_files:
  - zircon_runtime/src/navigation/mod.rs
  - zircon_runtime/src/navigation/module.rs
  - zircon_runtime/src/navigation/runtime.rs
  - zircon_runtime/src/navigation/runtime/avoidance.rs
  - zircon_runtime/src/navigation/runtime/baked_mesh.rs
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
  - dev/godot/modules/navigation_2d
  - dev/godot/modules/navigation_3d
tests:
  - cargo check -p zircon_runtime --lib --message-format short --color never
  - cargo test -p zircon_runtime --lib builtin_host_modules_register_gameplay_capabilities --message-format short --color never
  - cargo test -p zircon_runtime --lib vampire_example_manifest_scene_and_scripts_are_importable --message-format short --color never
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs::runtime_navigation_boundary_file_set_requires_doc_update
  - zircon_runtime/src/navigation/runtime/tests.rs::tick_world_agent_moves_only_selected_agent_and_avoids_local_colliders
  - "pending: cargo test -p zircon_runtime --lib navigation --locked"
doc_type: module-detail
---

# Runtime Navigation Module

## Purpose

`zircon_runtime::navigation` is the built-in lightweight navigation module used by standalone runtime sessions. It loads project-authored baked navmesh assets and exposes enough runtime pathing for project gameplay to move agents through the scene without requiring the external navigation plugin workspace to be linked into the executable.

The module registers as `NavigationModule` and provides the runtime navigation service consumed by gameplay host functions.

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
- `runtime/baked_mesh.rs` owns baked polygon storage, adjacency, A* pathfinding, sample-position, raycast, and path result helpers.
- `runtime/world_scan.rs` owns ECS component scanning for runtime agents, nearby agents, and runtime obstacles.
- `runtime/avoidance.rs` owns local obstacle and agent separation.
- `runtime/math.rs` owns shared navigation math helpers.
- `runtime/tests.rs` owns focused fallback runtime behavior tests.

Gameplay calls `nav_move_towards_entity(...)`, which writes the destination agent component to the mover and invokes the navigation runtime. This keeps enemy chase behavior tied to the baked scene instead of raw per-frame translation.

## Scope

This module is a runtime-owned bridge for project examples and standalone gameplay validation. The fuller Recast-backed plugin stack under `zircon_plugins/navigation` remains the long-term owner for advanced baking, Detour/TileCache integration, editor authoring, and plugin catalog capability reporting.

The current runtime navigation module does not replace that plugin. It gives `zircon_runtime` an independent, minimal, testable pathfinding capability so examples such as `examples/vampire` can run without requiring plugin-workspace runtime composition.

## Runtime 14 Boundary Judgment

Runtime 14 corrected the earlier "thin navigation module" assumption. The module now has 9 Rust owner files after the folder-backed runtime owner split, and `runtime.rs` remains the manager owner for real fallback behavior: baked navmesh loading, nearest-position sampling, pathfinding, raycast checks, world-agent ticking, obstacle collection, and local avoidance.

The crate-root seat remains intentional. `core::framework::navigation` owns contracts and DTOs, while `zircon_runtime::navigation` owns the built-in fallback implementation used when the external Recast-backed plugin stack is not linked into the process.

Godot remains a useful layering reference, but the verified source paths are `dev/godot/modules/navigation_2d` and `dev/godot/modules/navigation_3d`, not a single `modules/navigation` directory. Zircon should also avoid adopting "server" terminology here because this module is not a network service.

The Runtime 14 judgement is:

- Keep `navigation` at crate root as a self-contained runtime fallback.
- Do not split behavior into more root-level navigation families unless baking/editor/Recast ownership moves into `zircon_runtime`, which is currently a non-goal.
- Keep the folder-backed runtime owner split documented whenever `runtime/avoidance.rs`, `runtime/baked_mesh.rs`, `runtime/world_scan.rs`, `runtime/state.rs`, `runtime/math.rs`, or `runtime/tests.rs` changes ownership.
- If future code adds new navigation behavior files, document whether they extend the fallback runtime or belong in `zircon_plugins/navigation`.
`runtime_navigation_boundary_file_set_requires_doc_update` asserts the current file set and forces new behavior files to update this document before expanding the fallback runtime.
