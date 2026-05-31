---
related_code:
  - zircon_plugins/physics/plugin.toml
  - zircon_plugins/physics/runtime/src/backend.rs
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/src/module.rs
  - zircon_plugins/physics/runtime/src/manager.rs
  - zircon_plugins/physics/runtime/src/query_contact.rs
  - zircon_plugins/physics/runtime/src/scene_hook.rs
  - zircon_runtime/src/core/framework/physics/manager.rs
  - zircon_runtime/src/core/framework/physics/scene_step_result.rs
  - zircon_runtime/src/core/framework/physics/world_step_plan.rs
  - zircon_runtime/src/core/manager/service_names.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/plugin/scene_hook/mod.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
implementation_files:
  - zircon_plugins/physics/plugin.toml
  - zircon_plugins/physics/runtime/src/backend.rs
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/src/module.rs
  - zircon_plugins/physics/runtime/src/manager.rs
  - zircon_plugins/physics/runtime/src/query_contact.rs
  - zircon_plugins/physics/runtime/src/scene_hook.rs
  - zircon_runtime/src/core/framework/physics/manager.rs
  - zircon_runtime/src/core/framework/physics/scene_step_result.rs
  - zircon_runtime/src/core/framework/physics/world_step_plan.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
plan_sources:
  - user: 2026-05-03 继续补独立插件缺口
  - user: 2026-05-08 继续周边设施与插件能力完善计划
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
  - .codex/plans/ZirconEngine 周边设施与插件能力完善计划.md
  - docs/superpowers/plans/2026-05-03-physics-animation-aggressive-plugin-migration.md
tests:
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract.rs
  - zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract/mod.rs
  - zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract/contact.rs
  - zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract/query.rs
  - zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract/step.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - cargo test --manifest-path "zircon_plugins/Cargo.toml" -p zircon_plugin_physics_runtime --test physics_manager_runtime_contract contract::step::builtin_fixed_step_uses_live_world_records_before_node_cache_flush --locked --quiet -- --exact --nocapture
  - zircon_runtime/src/tests/extensions/animation_physics_absorption.rs
  - zircon_runtime/src/tests/extensions/manager_handles.rs
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked --test physics_manager_runtime_contract --target-dir target\codex-shared-a
  - cargo check --manifest-path zircon_plugins/Cargo.toml --locked --target-dir target\codex-shared-a
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --tests --locked --quiet (blocked: unrelated active scene world/ECS compile errors)
  - cargo test -p zircon_runtime --locked --lib --target-dir target\codex-shared-a
  - 2026-05-31: cargo test --manifest-path .\zircon_plugins\physics\runtime\Cargo.toml physics_registration_contributes_runtime_module --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-authoring-runtime-metadata --color never --quiet (red before linked capability-status metadata, then passed with existing runtime warnings)
  - 2026-05-31: cargo test --manifest-path .\Cargo.toml -p zircon_runtime --lib runtime_experimental_plugin_toml_matches_catalog_partial_metadata --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-authoring-runtime-metadata --color never --quiet (passed with existing runtime warnings)
doc_type: module-detail
---

# Physics Runtime Plugin

`zircon_plugins/physics/runtime` owns the concrete physics runtime after the hard cutover. The crate provides the `PhysicsModule` descriptor, the plugin-local `PhysicsDriver`, the explicit backend selector, the `DefaultPhysicsManager` fallback/backend state, ray/query and contact helpers, and the scene hook that runs physics at `SystemStage::FixedUpdate`.

`zircon_runtime` no longer exports `zircon_runtime::physics` and does not depend on the plugin crate. Runtime keeps only neutral contracts under `zircon_runtime::core::framework::physics`, manager service names/resolvers under `zircon_runtime::core::manager`, scene ECS state, and the generic scene hook protocol.

## Runtime Boundary

- The plugin contributes the lifecycle module through `RuntimeExtensionRegistry::register_module(module_descriptor())`.
- The plugin contributes tick behavior through `RuntimeExtensionRegistry::register_scene_hook(scene_hook_registration())`.
- Static `zircon_plugins/physics/plugin.toml`, the linked runtime package manifest, and `RuntimePluginDescriptor::builtin_catalog()` all classify Physics as category `runtime`, maturity `experimental`, with partial status rows for `runtime.plugin.physics` and `runtime.capability.physics.raycast`. This keeps package/export metadata consistent without promoting Jolt or full physics parity.
- `PhysicsSceneRuntimeHook` resolves `PhysicsManagerHandle` through the runtime manager resolver and calls `PhysicsManager::tick_scene_world(...)`.
- `DefaultPhysicsManager` owns settings persistence, per-world accumulator state, sync snapshots, ray-cast fallback, and contact fallback.
- `DefaultPhysicsManager::advance_clock(...)` now fills `PhysicsWorldStepPlan.interpolation_alpha` from the remaining fixed-step accumulator, clamped to `0.0..=1.0`.
- Builtin fixed-step integration and world sync enumerate `World::node_records()` instead of the deferred `World::nodes()` cache, so `FixedUpdate` observes bodies and colliders spawned or mutated before the next `PostUpdate` node-cache refresh.
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
- `PhysicsWorldStepPlan` carries `steps`, `step_seconds`, `remaining_seconds`, and `interpolation_alpha`; the alpha is neutral visual interpolation metadata for runtime/editor consumers and is zero when the backend cannot step.
- `PHYSICS_MANAGER_NAME` remains the stable service name consumed by runtime/editor callers.

The plugin can evolve Jolt or another backend behind `DefaultPhysicsManager` or a plugin-owned service without reintroducing `zircon_runtime::physics`.

## Validation Evidence

- Previous hard-cutover evidence: `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked --test physics_manager_runtime_contract --target-dir target\codex-shared-a` passed with 21 plugin contract tests before the backend selector seam was added.
- Previous hard-cutover evidence: `cargo check --manifest-path zircon_plugins/Cargo.toml --locked --target-dir target\codex-shared-a` passed for the independent plugin workspace with physics included but still outside the root workspace.
- Previous hard-cutover evidence: `cargo test -p zircon_runtime --locked --lib --target-dir target\codex-shared-a` passed with 767 runtime lib tests, validating scene hook dispatch, manager contracts, and hard-cutover structural assertions without depending on the plugin crate.
- Current backend selector seam: `rustfmt --edition 2021` passed for the touched physics runtime source and test files.
- Current backend selector seam: `cargo check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_physics_runtime --tests --locked --target-dir "target\codex-shared-a"` is blocked before physics test execution by unrelated active renderer code in `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs`, where the call to `render_compiled_scene(...)` supplies 10 arguments while the callee takes 8.
- Current interpolation-alpha seam: `cargo check --manifest-path "zircon_plugins/Cargo.toml" -p zircon_plugin_physics_runtime --tests --locked --quiet` is blocked before physics test execution by unrelated active scene world/ECS errors: `rebuild_fixed_component_presence_for_entity` visibility and missing `flush_pending_scene_systems_if_ready` call sites.
- Current live-world fixed-step seam: `cargo test --manifest-path "zircon_plugins/Cargo.toml" -p zircon_plugin_physics_runtime --test physics_manager_runtime_contract contract::step::builtin_fixed_step_uses_live_world_records_before_node_cache_flush --locked --quiet -- --exact --nocapture` passed after confirming the regression failed against the stale node-cache path.
