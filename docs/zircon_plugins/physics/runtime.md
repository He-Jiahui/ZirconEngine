---
related_code:
  - zircon_plugins/physics/runtime/src/backend.rs
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/src/module.rs
  - zircon_plugins/physics/runtime/src/manager.rs
  - zircon_plugins/physics/runtime/src/query_contact.rs
  - zircon_plugins/physics/runtime/src/scene_hook.rs
  - zircon_runtime/src/core/framework/physics/manager.rs
  - zircon_runtime/src/core/framework/physics/scene_step_result.rs
  - zircon_runtime/src/core/manager/service_names.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/plugin/scene_hook/mod.rs
  - zircon_runtime/src/scene/module/world_driver.rs
implementation_files:
  - zircon_plugins/physics/runtime/src/backend.rs
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/src/module.rs
  - zircon_plugins/physics/runtime/src/manager.rs
  - zircon_plugins/physics/runtime/src/query_contact.rs
  - zircon_plugins/physics/runtime/src/scene_hook.rs
  - zircon_runtime/src/core/framework/physics/manager.rs
  - zircon_runtime/src/core/framework/physics/scene_step_result.rs
plan_sources:
  - user: 2026-05-03 继续补独立插件缺口
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
  - docs/superpowers/plans/2026-05-03-physics-animation-aggressive-plugin-migration.md
tests:
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract.rs
  - zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract/mod.rs
  - zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract/contact.rs
  - zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract/query.rs
  - zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract/step.rs
  - zircon_runtime/src/tests/extensions/animation_physics_absorption.rs
  - zircon_runtime/src/tests/extensions/manager_handles.rs
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked --test physics_manager_runtime_contract --target-dir target\codex-shared-a
  - cargo check --manifest-path zircon_plugins/Cargo.toml --locked --target-dir target\codex-shared-a
  - cargo test -p zircon_runtime --locked --lib --target-dir target\codex-shared-a
doc_type: module-detail
---

# Physics Runtime Plugin

`zircon_plugins/physics/runtime` owns the concrete physics runtime after the hard cutover. The crate provides the `PhysicsModule` descriptor, the plugin-local `PhysicsDriver`, the explicit backend selector, the `DefaultPhysicsManager` fallback/backend state, ray/query and contact helpers, and the scene hook that runs physics at `SystemStage::FixedUpdate`.

`zircon_runtime` no longer exports `zircon_runtime::physics` and does not depend on the plugin crate. Runtime keeps only neutral contracts under `zircon_runtime::core::framework::physics`, manager service names/resolvers under `zircon_runtime::core::manager`, scene ECS state, and the generic scene hook protocol.

## Runtime Boundary

- The plugin contributes the lifecycle module through `RuntimeExtensionRegistry::register_module(module_descriptor())`.
- The plugin contributes tick behavior through `RuntimeExtensionRegistry::register_scene_hook(scene_hook_registration())`.
- `PhysicsSceneRuntimeHook` resolves `PhysicsManagerHandle` through the runtime manager resolver and calls `PhysicsManager::tick_scene_world(...)`.
- `DefaultPhysicsManager` owns settings persistence, per-world accumulator state, sync snapshots, ray-cast fallback, and contact fallback.
- `backend.rs` maps `PhysicsSettings` into the plugin-local runtime backend state. Only explicit `backend = "builtin"` activates the builtin fallback; unavailable backends do not silently fall through to builtin behavior.
- `zircon_runtime::scene::WorldDriver` dispatches installed hooks by schedule stage and contains no physics-specific logic.

## Backend Selection

The current runtime has one real executable backend, `builtin`, and one unavailable native slot, `jolt`.

- `backend = "builtin"` with `simulation_mode = Simulate` runs fixed-step writeback, syncs the world snapshot, and produces fallback query/contact data.
- `backend = "builtin"` with `simulation_mode = QueryOnly` syncs the sanitized world snapshot for ray/contact queries but records a zero-step plan and does not move rigid bodies.
- `backend = "jolt"` remains unavailable even when the feature flag is enabled, because no native Jolt runtime is linked yet.
- Unknown, blank, or `unconfigured` backends report `PhysicsBackendState::Unavailable` and clear the manager's synchronized world/contact state for the tick instead of using builtin as an accidental downgrade path.
- `simulation_mode = Disabled` reports `PhysicsBackendState::Disabled` and records no executable backend.

## Framework Contract

Runtime framework contracts are intentionally concrete-free:

- `PhysicsManager::tick_scene_world(...)` defines the manager-side scene tick capability.
- `PhysicsSceneStepResult` returns the step plan and contact events without exposing plugin-owned implementation details.
- `PHYSICS_MANAGER_NAME` remains the stable service name consumed by runtime/editor callers.

The plugin can evolve Jolt or another backend behind `DefaultPhysicsManager` or a plugin-owned service without reintroducing `zircon_runtime::physics`.

## Validation Evidence

- Previous hard-cutover evidence: `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked --test physics_manager_runtime_contract --target-dir target\codex-shared-a` passed with 21 plugin contract tests before the backend selector seam was added.
- Previous hard-cutover evidence: `cargo check --manifest-path zircon_plugins/Cargo.toml --locked --target-dir target\codex-shared-a` passed for the independent plugin workspace with physics included but still outside the root workspace.
- Previous hard-cutover evidence: `cargo test -p zircon_runtime --locked --lib --target-dir target\codex-shared-a` passed with 767 runtime lib tests, validating scene hook dispatch, manager contracts, and hard-cutover structural assertions without depending on the plugin crate.
- Current backend selector seam: `rustfmt --edition 2021` passed for the touched physics runtime source and test files.
- Current backend selector seam: `cargo check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_physics_runtime --tests --locked --target-dir "target\codex-shared-a"` is blocked before physics test execution by unrelated active renderer code in `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs`, where the call to `render_compiled_scene(...)` supplies 10 arguments while the callee takes 8.
