---
related_code:
  - zircon_runtime/Cargo.toml
  - zircon_plugins/physics/runtime/Cargo.toml
  - zircon_plugins/physics/runtime/build.rs
  - zircon_plugins/physics/runtime/src/backend/mod.rs
  - zircon_plugins/physics/runtime/src/backend/contract.rs
  - zircon_plugins/physics/runtime/src/backend/selection.rs
  - zircon_plugins/physics/runtime/src/backend/jolt/mod.rs
  - zircon_plugins/physics/runtime/src/backend/jolt/layers.rs
  - zircon_plugins/physics/runtime/src/backend/jolt/conversion.rs
  - zircon_plugins/physics/runtime/src/backend/jolt/native_world.rs
  - zircon_plugins/physics/runtime/src/backend/jolt/runtime.rs
  - zircon_plugins/physics/runtime/src/backend/builtin/runtime.rs
  - zircon_plugins/physics/runtime/src/backend/builtin/step.rs
  - zircon_plugins/physics/runtime/src/manager.rs
  - zircon_plugins/physics/runtime/src/manager/change_detection.rs
  - zircon_plugins/physics/runtime/src/manager/command_buffer.rs
  - zircon_plugins/physics/runtime/src/manager/jolt_world.rs
  - zircon_runtime/src/core/framework/physics/manager.rs
  - docs/zircon_plugins/physics/runtime.md
implementation_files:
  - zircon_plugins/physics/runtime/Cargo.toml
  - zircon_plugins/physics/runtime/build.rs
  - zircon_plugins/physics/runtime/src/backend/mod.rs
  - zircon_plugins/physics/runtime/src/backend/contract.rs
  - zircon_plugins/physics/runtime/src/backend/selection.rs
  - zircon_plugins/physics/runtime/src/backend/jolt/mod.rs
  - zircon_plugins/physics/runtime/src/backend/jolt/layers.rs
  - zircon_plugins/physics/runtime/src/backend/jolt/conversion.rs
  - zircon_plugins/physics/runtime/src/backend/jolt/native_world.rs
  - zircon_plugins/physics/runtime/src/backend/jolt/runtime.rs
  - zircon_plugins/physics/runtime/src/backend/builtin/runtime.rs
  - zircon_plugins/physics/runtime/src/backend/builtin/step.rs
  - zircon_plugins/physics/runtime/src/manager.rs
  - zircon_plugins/physics/runtime/src/manager/change_detection.rs
  - zircon_plugins/physics/runtime/src/manager/command_buffer.rs
  - zircon_plugins/physics/runtime/src/manager/jolt_world.rs
plan_sources:
  - docs/plans/zircon_plugins/03-physics.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
  - dev/Fyrox/fyrox-impl/Cargo.toml
  - dev/godot/modules/godot_physics_3d
  - dev/godot/modules/jolt_physics
  - dev/bevy/crates
tests:
  - tests/acceptance/plugins-03-physics-backend-trait-builtin.md
  - tests/acceptance/plugins-03-physics-jolt-backend.md
  - tests/acceptance/plugins-03-physics-change-detection-command-buffer.md
  - zircon_plugins/physics/runtime/src/backend/tests/builtin_contract.rs
  - zircon_plugins/physics/runtime/src/backend/tests/jolt_contract.rs
  - physics_backend_generation_rejects_destroyed_body_after_slot_reuse
  - builtin_physics_backend_trait_steps_active_bodies_and_answers_queries
  - builtin_constraint_gap_is_a_typed_unsupported_error
  - zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract
  - empty_jolt_feature_slot_reports_unavailable_not_ready
  - unavailable_jolt_backend_does_not_fallback_to_builtin_scene_tick
  - linked_jolt_backend_reports_ready
  - linked_jolt_backend_ticks_scene_without_builtin_fallback
  - jolt_box_stack_settles_deterministically
  - jolt_creates_box_sphere_and_capsule_bodies
  - unchanged_bodies_skip_sync
  - force_applied_outside_fixed_update_lands_next_step
  - jolt_queued_force_lands_on_next_fixed_step
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked
doc_type: module-detail
---

# Physics Plugin Options

## Decision

The physics runtime keeps the plugin-owned `builtin` fallback and now links Jolt as the selected native backend behind the plugin crate's `backend-jolt` feature. No Rapier dependency is introduced, and no concrete physics library is added to `zircon_runtime`.

`zircon_runtime/Cargo.toml` keeps `backend-jolt` as profile vocabulary only. `zircon_plugins/physics/runtime/Cargo.toml` owns the executable gate and maps it to optional `joltc-sys = 0.3.1+Jolt-5.0.0`. Enabling the plugin feature makes `backend = "jolt"` Ready; disabling it makes the same setting explicitly Unavailable. Neither path silently downgrades to builtin stepping, and the retired `jolt` Cargo feature has no alias.

The native dependency and build policy remain plugin-owned. On non-Apple Unix targets the plugin build script links the C++ standard library required by the vendored JoltC build; MSVC uses its native runtime. Bindgen requires libclang at build time, with `LIBCLANG_PATH` set explicitly when the toolchain cannot discover it.

## Current Baseline

`zircon_plugins/physics/runtime` owns the concrete runtime module, a typed `PhysicsBackend` contract with generation-checked handles, `BuiltinPhysicsBackend`, `JoltPhysicsBackend`, manager service implementation, per-world native Jolt state, submitted-state change detection, a bounded external-write command buffer, world sync, fallback queries/events, settings validation, the `physics.step` FixedUpdate system, and the `physics.sync_to_scene` FixedPostUpdate writeback system. Both anchors belong to `physics.main`.

The runtime crate owns only neutral contracts under `zircon_runtime::core::framework::physics`, manager service names/resolution, scene ECS data, and the generic scene-hook protocol. This boundary remains fixed for every option below.

## Option Matrix

| Option | Decision | Reference engine evidence | Runtime boundary | Cost and fallback |
|---|---|---|---|---|
| Jolt native backend | Selected and linked behind the plugin `backend-jolt` feature for Box/Sphere/Capsule shape creation, body lifecycle, commands, step, and active-state writeback. | Godot keeps its in-tree physics module and Jolt module side by side; the local Godot tree informed layer/filter and ownership boundaries. JoltC HelloWorld and headers informed native lifecycle ordering. | All JoltC calls live under `zircon_plugins/physics/runtime/src/backend/jolt`; the manager owns one persistent native world per scene world. `zircon_runtime` sees only neutral DTOs and `PhysicsManager`. | Native query/event/constraint parity remains later milestone work. Feature-off reports Unavailable rather than falling back to builtin. |
| Rapier backend | Not selected for the primary path. Keep as a fallback candidate only if Jolt integration is blocked. | Fyrox uses `rapier2d` / `rapier3d`, so Rapier is proven for Rust-engine integration, but it would not match the existing Jolt feature slot. | Would still be plugin-owned through the same `PhysicsManager` contract. | Lower Rust binding cost than Jolt, but adds a separate solver vocabulary and duplicates the current backend-selection shape. |
| Extend builtin fallback | Retained as the default no-feature executable behavior, not the long-term full solver strategy. | Bevy core has no built-in physics engine; plugin-owned behavior is acceptable. Unreal/Godot show that full built-in physics is a major subsystem, not a small fallback. | Stays in `zircon_plugins/physics/runtime/src/backend/builtin`; runtime framework DTOs remain neutral. | Good for deterministic tests, query fallback, and basic authoring flows. It must not absorb ragdoll, constraint-solver, broadphase, or native collision parity work indefinitely. |

## Jolt Feature Ruling

The two `backend-jolt` feature declarations have distinct responsibilities:

- `zircon_runtime/Cargo.toml` keeps the feature as a runtime/profile vocabulary hook only and has no Jolt dependency.
- `zircon_plugins/physics/runtime/Cargo.toml` activates optional `joltc-sys` and compiles `backend/jolt/**`.
- `backend = "jolt"` with the plugin feature enabled reports Ready and uses the native world.
- `backend = "jolt"` with the feature disabled reports that the feature is not enabled.
- Neither path may select builtin stepping implicitly.

The anchor tests are `linked_jolt_backend_reports_ready`, `linked_jolt_backend_ticks_scene_without_builtin_fallback`, `empty_jolt_feature_slot_reports_unavailable_not_ready`, and `unavailable_jolt_backend_does_not_fallback_to_builtin_scene_tick`; conditional compilation runs the appropriate pair for each feature state.

## Implemented Scope And Backlog

The first native milestone now includes dependency acquisition, platform link policy, allocator/factory/type registration, collision-layer filters, Box/Sphere/Capsule conversion, body lifecycle, submitted-state change detection, a bounded manager command buffer, deterministic stepping, active-state reads, and manager writeback. External commands target world/entity identities, are resolved to current generation-checked handles immediately before a real FixedUpdate, and never mutate the scene at queue time. WSL M1 completion tests pass 46/46 with Jolt enabled and 43/43 with the feature disabled; focused Windows MSVC native tests pass 2/2.

The next work stays explicit:

- M2/M3: native ray/overlap/shape-cast parity plus contact and trigger event draining.
- M4/M5: six constraint families, skeleton binding, and ragdoll behavior.
- Additional platform CI and deterministic tuning beyond the current WSL and Windows evidence.

M1-T4 is accepted: unchanged bodies skip backend synchronization, structural changes recreate bodies, and queued force/impulse/velocity/teleport/type writes land on the next real step for both builtin and Jolt. Until later milestones land, manager queries continue through the synchronized neutral snapshot; the code does not claim native query, event, constraint, or ragdoll parity.
