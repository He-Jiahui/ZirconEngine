---
related_code:
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/module.rs
  - zircon_plugins/animation/runtime/src/manager.rs
  - zircon_plugins/animation/runtime/src/sequence.rs
  - zircon_plugins/animation/runtime/src/scene_hook.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - zircon_runtime/src/core/framework/animation/sequence_apply_report.rs
  - zircon_runtime/src/core/manager/service_names.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/plugin/scene_hook/mod.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/level_system.rs
implementation_files:
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/module.rs
  - zircon_plugins/animation/runtime/src/manager.rs
  - zircon_plugins/animation/runtime/src/sequence.rs
  - zircon_plugins/animation/runtime/src/scene_hook.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - zircon_runtime/src/core/framework/animation/sequence_apply_report.rs
  - zircon_runtime/src/scene/level_system.rs
plan_sources:
  - user: 2026-05-03 继续补独立插件缺口
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
  - docs/superpowers/plans/2026-05-03-physics-animation-aggressive-plugin-migration.md
tests:
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract.rs
  - zircon_runtime/src/tests/extensions/animation_physics_absorption.rs
  - zircon_runtime/src/tests/extensions/manager_handles.rs
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --locked --test runtime_physics_animation_tick_contract --target-dir target\codex-shared-a
  - cargo check --manifest-path zircon_plugins/Cargo.toml --locked --target-dir target\codex-shared-a
  - cargo test -p zircon_runtime --locked --lib --target-dir target\codex-shared-a
doc_type: module-detail
---

# Animation Runtime Plugin

`zircon_plugins/animation/runtime` owns the concrete animation runtime after the hard cutover. The crate provides the `AnimationModule` descriptor, the plugin-local `AnimationDriver`, the `DefaultAnimationManager` evaluator/sampler, sequence property writeback, and the scene hook that runs animation at `SystemStage::LateUpdate`.

`zircon_runtime` no longer exports `zircon_runtime::animation` and does not depend on the plugin crate. Runtime keeps only neutral contracts under `zircon_runtime::core::framework::animation`, manager service names/resolvers under `zircon_runtime::core::manager`, scene ECS state, and generic scene hook scheduling.

## Runtime Boundary

- The plugin contributes the lifecycle module through `RuntimeExtensionRegistry::register_module(module_descriptor())`.
- The plugin contributes tick behavior through `RuntimeExtensionRegistry::register_scene_hook(scene_hook_registration())`.
- `AnimationSceneRuntimeHook` resolves `AnimationManagerHandle`, advances scene player clocks, loads animation assets through `ProjectAssetManager`, and records pose/playback runtime state on `LevelSystem`.
- `DefaultAnimationManager` owns playback settings persistence, graph evaluation, state-machine evaluation, clip pose sampling, and sequence-to-world application.
- `zircon_runtime::scene::WorldDriver` dispatches installed hooks by schedule stage and contains no animation-specific logic.

## Framework Contract

Runtime framework contracts are intentionally concrete-free:

- `AnimationManager::apply_sequence_to_world(...)` defines the manager-side sequence writeback capability.
- `AnimationSequenceApplyReport` reports applied and missing tracks without exposing plugin-owned sequence implementation details.
- `ANIMATION_MANAGER_NAME` remains the stable service name consumed by runtime/editor callers.

The plugin can evolve graph blending, state-machine semantics, and importer-driven animation assets without reintroducing `zircon_runtime::animation`.

## Validation Evidence

- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --locked --test runtime_physics_animation_tick_contract --target-dir target\codex-shared-a` passed with 7 plugin contract tests.
- `cargo check --manifest-path zircon_plugins/Cargo.toml --locked --target-dir target\codex-shared-a` passed for the independent plugin workspace with animation included but still outside the root workspace.
- `cargo test -p zircon_runtime --locked --lib --target-dir target\codex-shared-a` passed with 767 runtime lib tests, validating scene hook dispatch, manager contracts, and hard-cutover structural assertions without depending on the plugin crate.
