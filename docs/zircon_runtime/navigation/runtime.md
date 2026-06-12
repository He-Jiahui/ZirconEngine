---
related_code:
  - zircon_runtime/src/navigation/mod.rs
  - zircon_runtime/src/navigation/module.rs
  - zircon_runtime/src/navigation/runtime.rs
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
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
plan_sources:
  - user: 2026-06-11 vampire roguelite runtime example and screenshot validation
tests:
  - cargo check -p zircon_runtime --lib --message-format short --color never
  - cargo test -p zircon_runtime --lib builtin_host_modules_register_gameplay_capabilities --message-format short --color never
  - cargo test -p zircon_runtime --lib vampire_example_manifest_scene_and_scripts_are_importable --message-format short --color never
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

Gameplay calls `nav_move_towards_entity(...)`, which writes the destination agent component to the mover and invokes the navigation runtime. This keeps enemy chase behavior tied to the baked scene instead of raw per-frame translation.

## Scope

This module is a runtime-owned bridge for project examples and standalone gameplay validation. The fuller Recast-backed plugin stack under `zircon_plugins/navigation` remains the long-term owner for advanced baking, Detour/TileCache integration, editor authoring, and plugin catalog capability reporting.

The current runtime navigation module does not replace that plugin. It gives `zircon_runtime` an independent, minimal, testable pathfinding capability so examples such as `examples/vampire` can run without requiring plugin-workspace runtime composition.
