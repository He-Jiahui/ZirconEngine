# Physics Animation Aggressive Plugin Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move physics and animation runtime execution from hard-coded `zircon_runtime` scene ticks to plugin-registered scene hooks while keeping shared contracts, scene ECS authority, and asset/property-path DTOs in `zircon_runtime`.

**Architecture:** `zircon_runtime` adds a neutral scene hook protocol to `RuntimeExtensionRegistry`; `WorldDriver` dispatches registered hooks by `SystemStage`; `zircon_plugins/physics/runtime` and `zircon_plugins/animation/runtime` become the concrete owners of physics/animation tick hooks and module descriptors. This is a hard cutover: no compatibility re-exports, shim modules, forwarding roots, or built-in fallback registrations survive once their callers move.

**Tech Stack:** Rust, Cargo, `zircon_runtime`, independent `zircon_plugins` workspace, module descriptors, runtime plugin registration reports, scene ECS, `target/codex-shared-a` validation target.

---

## Architecture Notes

- Owner boundary: `zircon_runtime::plugin` owns hook registration contracts; `zircon_runtime::scene` owns hook dispatch and world state; `zircon_runtime::core::framework::{physics,animation}` owns shared manager DTOs; `zircon_plugins/{physics,animation}/runtime` owns concrete manager/module/hook behavior.
- Reference grounding: Bevy `Plugin::build` and `app.add_systems(...)` support plugin-contributed schedule behavior; Fyrox `Plugin::{register,init,update,on_deinit}` supports lifecycle-aware plugin execution; Godot physics server/backend registration supports runtime core owning scene state while external subsystem providers own concrete backend behavior.
- Hard-cutover target: after Milestone 4, `WorldDriver` contains no direct `crate::physics` or `crate::animation` tick behavior, runtime tests activate physics/animation through plugin registration reports, and old runtime module descriptor files remain deleted.
- Validation principle: implementation slices may add tests and docs without immediate build loops; each milestone has a named testing stage with scoped Cargo commands and correction loops.

## File Map

- Create: `zircon_runtime/src/plugin/scene_hook/mod.rs` for `SceneRuntimeHook`, `SceneRuntimeHookDescriptor`, `SceneRuntimeHookRegistration`, and `SceneRuntimeHookContext`.
- Modify: `zircon_runtime/src/plugin/mod.rs` to export the scene hook protocol.
- Modify: `zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs` to store hook registrations.
- Modify: `zircon_runtime/src/plugin/extension_registry/register.rs` to register hooks and reject duplicate ids.
- Modify: `zircon_runtime/src/plugin/extension_registry/access.rs` to expose ordered hooks.
- Modify: `zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs` to merge hook registrations.
- Modify: `zircon_runtime/src/plugin/extension_registry_error.rs` to add duplicate and invalid hook diagnostics.
- Modify: `zircon_runtime/src/scene/module/world_driver.rs` to dispatch hook stages from registry state and remove hard-coded physics/animation behavior.
- Modify: `zircon_runtime/src/scene/level_system.rs` to expose runtime-state read/write helpers needed by hooks without exposing editor authority.
- Modify: `zircon_plugins/physics/runtime/src/lib.rs` and `zircon_plugins/physics/runtime/src/module.rs` to register the physics scene hook alongside the module descriptor.
- Modify: `zircon_plugins/animation/runtime/src/lib.rs` and `zircon_plugins/animation/runtime/src/module.rs` to register the animation scene hook alongside the module descriptor.
- Move/update: `zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract.rs` and `zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract/**` to activate through plugin registration reports and installed scene hooks.
- Modify: `zircon_runtime/src/tests/plugin_extensions/extension_registry.rs` for registry and duplicate hook coverage.
- Modify: `zircon_runtime/src/tests/extensions/animation_physics_absorption.rs` to assert concrete behavior is plugin-owned, not runtime-owned.
- Modify: `docs/assets-and-rendering/runtime-physics-animation-assets.md` to document the hook contract, ownership split, and validation evidence.
- Update: `.codex/sessions/20260503-1538-physics-animation-plugin-runtime.md` after each milestone with touched modules, commands, and blockers.

## Milestone 1: Scene Hook Foundation

### Implementation Slices

- [x] Add `zircon_runtime/src/plugin/scene_hook/mod.rs` with a minimal hook protocol:
  - `SceneRuntimeHookDescriptor { id, plugin_id, stage, order }`
  - `SceneRuntimeHookRegistration { descriptor, hook: Arc<dyn SceneRuntimeHook> }`
  - `SceneRuntimeHookContext<'a> { core: &'a CoreHandle, level: &'a LevelSystem, delta_seconds: Real }`
  - `SceneRuntimeHook::run(&self, context: SceneRuntimeHookContext<'_>) -> Result<(), CoreError>`
- [x] Export the hook protocol from `zircon_runtime/src/plugin/mod.rs` and keep root files structural only.
- [x] Extend `RuntimeExtensionRegistry` with `scene_hooks: Vec<SceneRuntimeHookRegistration>` and `scene_hooks()` access.
- [x] Add `register_scene_hook(...)` with validation that hook ids are non-empty, trimmed, prefixed by `plugin_id`, and unique.
- [x] Merge hook registrations in `RuntimePluginCatalog::{runtime_extensions,runtime_extensions_for_project}` preserving plugin registration order and sorting within a stage by `order` then `id`.
- [x] Replace `WorldDriver::tick_level(...)` hard-coded physics/animation calls with stage dispatch over registered hooks for `Schedule::default().stages`.
- [x] Add runtime registry tests for hook collection, duplicate rejection, invalid prefix rejection, and deterministic order.
- [x] Keep `LevelSystem` runtime state helpers narrow: hooks may read/write physics step/contact caches and animation pose/playback caches, but may not mutate editor state or bypass `with_world_mut`.

### Testing Stage

- [x] Run `cargo test -p zircon_runtime --locked --lib plugin_extensions::extension_registry --target-dir target\codex-shared-a`.
- [x] Covered scene-level hook dispatch in `cargo test -p zircon_runtime --locked --lib --target-dir target\codex-shared-a`.
- [x] Fix compile or test failures in the lowest shared layer first, then rerun the failed command.
- [x] Record command output and remaining risks in the active session note and docs.

## Milestone 2: Physics Plugin Hook Cutover

### Implementation Slices

- [x] Move physics tick behavior out of `WorldDriver` into `zircon_plugins/physics/runtime` as a `PhysicsSceneHook` running in `SystemStage::FixedUpdate`.
- [x] Ensure the hook resolves `PhysicsManagerHandle`, plans fixed steps, calls plugin-owned fallback integration/sync helpers, drains contacts, and records `PhysicsWorldStepPlan` plus contacts back into `LevelSystem`.
- [x] Move or duplicate only concrete physics implementation into the plugin crate; leave shared DTOs and scene components in runtime.
- [x] Stop exporting concrete physics manager/module behavior from `zircon_runtime::physics` once all direct consumers move.
- [x] Rewrite runtime physics tests to build `RuntimePluginRegistrationReport::from_plugin(&zircon_plugin_physics_runtime::runtime_plugin())`, register contributed modules, activate them, and tick through hook registration.
- [x] Add a disabled-plugin assertion that `LevelSystem::tick(...)` does not run physics when the physics plugin report is absent.
- [x] Re-run hard-cutover searches for `crate::physics::module`, `physics::DefaultPhysicsManager` in runtime tests, `pub use zircon_runtime::physics`, `compat`, `shim`, `bridge`, and `legacy` within the touched physics lane.

### Testing Stage

- [x] Run `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked --test physics_manager_runtime_contract --target-dir target\codex-shared-a`.
- [x] Run `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --locked --test runtime_physics_animation_tick_contract --target-dir target\codex-shared-a`.
- [x] Run `cargo check --manifest-path zircon_plugins/Cargo.toml --locked --target-dir target\codex-shared-a`.
- [x] Fix failures bottom-up and document whether any remaining failure is unrelated active-session baseline.

## Milestone 3: Animation Plugin Hook Cutover

### Implementation Slices

- [x] Move animation tick behavior out of `WorldDriver` into `zircon_plugins/animation/runtime` as an `AnimationSceneHook` running after physics in `SystemStage::LateUpdate`.
- [x] Keep `zircon_runtime::scene` authoritative for component storage and property-path mutation; the hook uses `LevelSystem::with_world_mut(...)` and runtime asset manager facades.
- [x] Move concrete animation evaluator, graph blend, state-machine transition runtime, and sequence writeback helpers into the animation plugin crate when they are no longer shared runtime contracts.
- [x] Stop exporting concrete animation manager/module behavior from `zircon_runtime::animation` once all direct consumers move.
- [x] Rewrite animation tick tests to activate through `zircon_plugin_animation_runtime::runtime_plugin()` registration reports.
- [x] Add a disabled-plugin assertion that animation sequence players do not advance and pose cache remains empty when the animation plugin report is absent.
- [x] Re-run hard-cutover searches for `crate::animation::module`, `animation::DefaultAnimationManager` in runtime tests, `pub use zircon_runtime::animation`, `compat`, `shim`, `bridge`, and `legacy` within the touched animation lane.

### Testing Stage

- [x] Run `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --locked --test runtime_physics_animation_tick_contract --target-dir target\codex-shared-a`.
- [x] Confirm `animation_assets_report_direct_references` is no longer the active runtime ownership gate; runtime lib hard-cutover tests cover the current boundary.
- [x] Run `cargo check --manifest-path zircon_plugins/Cargo.toml --locked --target-dir target\codex-shared-a`.
- [x] Fix failures bottom-up and document whether any remaining failure is unrelated active-session baseline.

## Milestone 4: Hard Cutover Cleanup And Documentation

### Implementation Slices

- [x] Delete runtime concrete physics/animation files that no longer own real shared contracts.
- [x] Remove stale imports from `WorldDriver`, runtime tests, `runtime_absorption` tests, and docs that imply runtime-owned concrete manager/module registration.
- [x] Update `docs/assets-and-rendering/runtime-physics-animation-assets.md` related-code headers, ownership section, scene tick runtime section, and tests list.
- [x] Update or archive the active coordination note with the final status and remaining blockers.
- [x] Run hard-cutover searches for old paths and migration-smell terms across touched subsystems.

### Testing Stage

- [x] Run `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --locked --test runtime_physics_animation_tick_contract --target-dir target\codex-shared-a`.
- [x] Run `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked --test physics_manager_runtime_contract --target-dir target\codex-shared-a`.
- [x] Run `cargo check --manifest-path zircon_plugins/Cargo.toml --locked --target-dir target\codex-shared-a`.
- [x] If scoped commands pass and shared APIs/manifests moved, run `cargo test -p zircon_runtime --locked --lib --target-dir target\codex-shared-a` before claiming runtime-package confidence.
- [x] Do not claim workspace green unless full workspace validation was run fresh and passed.

## Acceptance Checklist

- [x] Physics and animation tick behavior is activated only by linked plugin registration reports.
- [x] `WorldDriver` dispatches generic scene hooks and contains no domain-specific physics/animation tick logic.
- [x] `zircon_runtime` retains shared contracts, scene components, asset DTOs, property paths, and runtime state caches only.
- [x] `zircon_plugins/physics/runtime` owns physics concrete manager/module/hook behavior.
- [x] `zircon_plugins/animation/runtime` owns animation concrete manager/module/hook behavior.
- [x] Tests cover hook registration, duplicate rejection, deterministic stage ordering, disabled-plugin no-op behavior, physics tick, animation sequence writeback, graph blend, and state-machine timed transition.
- [x] Docs and session note state exactly what commands ran and what remains unresolved.
