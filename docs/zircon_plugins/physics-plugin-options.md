---
related_code:
  - zircon_runtime/Cargo.toml
  - zircon_plugins/physics/runtime/Cargo.toml
  - zircon_plugins/physics/runtime/src/backend.rs
  - zircon_plugins/physics/runtime/src/manager.rs
  - zircon_plugins/physics/runtime/src/manager/builtin_step.rs
  - zircon_plugins/physics/runtime/src/query_contact.rs
  - zircon_plugins/physics/runtime/src/trigger.rs
  - zircon_runtime/src/core/framework/physics/manager.rs
  - docs/zircon_plugins/physics/runtime.md
implementation_files:
  - zircon_plugins/physics/runtime/src/backend.rs
  - zircon_plugins/physics/runtime/src/manager.rs
  - zircon_plugins/physics/runtime/src/manager/builtin_step.rs
  - zircon_plugins/physics/runtime/src/query_contact.rs
  - zircon_plugins/physics/runtime/src/trigger.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
  - dev/Fyrox/fyrox-impl/Cargo.toml
  - dev/godot/modules/godot_physics_3d
  - dev/godot/modules/jolt_physics
  - dev/bevy/crates
tests:
  - zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract
  - empty_jolt_feature_slot_reports_unavailable_not_ready
  - unavailable_jolt_backend_does_not_fallback_to_builtin_scene_tick
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked
doc_type: module-detail
---

# Physics Plugin Options

## Decision

The physics runtime keeps the current plugin-owned `builtin` fallback as the only executable V1 backend and selects Jolt as the future native backend direction. No Rapier dependency is introduced, and no concrete physics library is added to `zircon_runtime`.

The existing empty `jolt = []` feature slots in `zircon_runtime/Cargo.toml` and `zircon_plugins/physics/runtime/Cargo.toml` stay for now as explicit backend-selection compatibility with authored settings, but they must remain unavailable until a real plugin-owned Jolt bridge is linked. Enabling the feature must not silently downgrade `backend = "jolt"` to builtin stepping.

## Current Baseline

`zircon_plugins/physics/runtime` is no longer an empty shell. It already owns the concrete runtime plugin module, manager service implementation, builtin rigid-body step, world sync, ray/shape query fallback, contact helpers, trigger events, settings validation, and the scene hook that runs at `SystemStage::FixedUpdate`.

The runtime crate owns only neutral contracts under `zircon_runtime::core::framework::physics`, manager service names/resolution, scene ECS data, and the generic scene-hook protocol. This boundary remains fixed for every option below.

## Option Matrix

| Option | Decision | Reference engine evidence | Runtime boundary | Cost and fallback |
|---|---|---|---|---|
| Jolt native backend | Selected future native direction, not linked in V1. Keep `jolt` feature slots unavailable until a real bridge exists. | Godot keeps its in-tree physics module and a Jolt module side by side; this matches the current builtin + future native slot shape. | Jolt code must live behind `zircon_plugins/physics/runtime`, expose only `PhysicsManager` behavior, and never become a `zircon_runtime` dependency. | Requires native build/link policy, deterministic settings, shape/joint parity, and platform CI. If unavailable, reports `PhysicsBackendState::Unavailable` rather than falling back to builtin. |
| Rapier backend | Not selected for the primary path. Keep as a fallback candidate only if Jolt integration is blocked. | Fyrox uses `rapier2d` / `rapier3d`, so Rapier is proven for Rust-engine integration, but it would not match the existing Jolt feature slot. | Would still be plugin-owned through the same `PhysicsManager` contract. | Lower Rust binding cost than Jolt, but adds a separate solver vocabulary and duplicates the current backend-selection shape. |
| Extend builtin fallback | Selected as the default executable V1 behavior, not the long-term full solver strategy. | Bevy core has no built-in physics engine; plugin-owned behavior is acceptable. Unreal/Godot show that full built-in physics is a major subsystem, not a small fallback. | Stays in `zircon_plugins/physics/runtime::manager`, `query_contact`, and `trigger`; runtime framework DTOs remain neutral. | Good for deterministic tests, query fallback, and basic authoring flows. It should not absorb ragdoll, constraint-solver, broadphase, or native collision parity work indefinitely. |

## Jolt Feature Slot Ruling

The two current `jolt = []` feature slots are retained under a strict unavailable-backend contract:

- `zircon_runtime/Cargo.toml` keeps the feature as a runtime/profile vocabulary hook only.
- `zircon_plugins/physics/runtime/Cargo.toml` keeps the feature as the plugin backend gate.
- `zircon_plugins/physics/runtime/src/backend.rs` keeps `JOLT_BACKEND_AVAILABLE = false` until real native linkage exists.
- `backend = "jolt"` with the feature enabled must report that no runtime Jolt backend is linked.
- `backend = "jolt"` with the feature disabled must report that the feature is not enabled.
- Neither path may select builtin stepping implicitly.

The anchor tests for this ruling are `empty_jolt_feature_slot_reports_unavailable_not_ready` and `unavailable_jolt_backend_does_not_fallback_to_builtin_scene_tick`.

## Implementation Backlog

The next Jolt implementation milestone must land as plugin-owned code and include:

- Native dependency acquisition and platform build policy.
- Shape vocabulary mapping from the existing neutral framework DTOs.
- Joint and skeleton-binding parity for the metadata already synchronized by the plugin.
- Query/contact parity for ray casts, overlap, shape cast, triggers, and contact events.
- Deterministic fallback behavior when the native backend cannot be loaded.
- Cargo validation through the plugin workspace, not by adding a direct `zircon_runtime` dependency.

Until that milestone exists, the builtin fallback remains the only executable backend and the Jolt option remains a visible but unavailable backend slot.
