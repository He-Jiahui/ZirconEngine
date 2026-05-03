---
related_code:
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/module.rs
  - zircon_runtime/src/animation/mod.rs
  - zircon_runtime/src/animation/runtime/mod.rs
  - zircon_runtime/src/animation/sequence/mod.rs
  - zircon_runtime/src/core/manager/service_names.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/tests/common/mod.rs
implementation_files:
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/module.rs
  - zircon_runtime/src/animation/mod.rs
  - zircon_runtime/src/animation/runtime/mod.rs
  - zircon_runtime/src/animation/sequence/mod.rs
plan_sources:
  - user: 2026-05-03 继续补独立插件缺口
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
tests:
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_runtime/src/tests/extensions/animation_physics_absorption.rs
  - zircon_runtime/src/tests/extensions/manager_handles.rs
  - zircon_runtime/tests/common/mod.rs
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime -p zircon_plugin_animation_runtime --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-independent-plugin-physics-animation-cutover --message-format short --color never
  - cargo check -p zircon_runtime --lib --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-physics-animation-cutover --message-format short --color never
doc_type: module-detail
---

# Animation Runtime Plugin

`zircon_plugins/animation/runtime` now owns the animation runtime module descriptor. Its `module.rs` declares `AnimationModule`, the plugin-local `AnimationDriver`, and the manager descriptors that create `DefaultAnimationManager` plus the canonical `AnimationManagerHandle`.

`zircon_runtime::animation` remains the current contract owner for clip/graph/state-machine evaluation, sequence property application, playback settings persistence, and `ANIMATION_PLAYBACK_CONFIG_KEY`. Runtime no longer exports `animation::module_descriptor`, `AnimationModule`, or `AnimationDriver`.

## Runtime Boundary

- The plugin crate contributes the lifecycle module through `RuntimeExtensionRegistry::register_module(module_descriptor())`.
- The canonical service name for `AnimationManagerHandle` still comes from `zircon_runtime::core::manager::ANIMATION_MANAGER_NAME`.
- The default manager type is still `zircon_runtime::animation::DefaultAnimationManager` until the manager/evaluator implementation moves behind the plugin boundary.
- Runtime tests that need a local animation module use explicit fixtures instead of a production runtime module descriptor.

## Remaining Migration

The module descriptor and driver are plugin-owned, but graph evaluation, clip pose sampling, state-machine transition logic, and sequence-to-world property writeback still live in `zircon_runtime::animation`. The final cutover should move those behavior paths into this plugin or a plugin-owned runtime service, leaving runtime with neutral animation DTOs, scene components, and manager access contracts only.

`cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime -p zircon_plugin_animation_runtime --lib ...` timed out during Windows test-binary build/link under concurrent Cargo load on 2026-05-03. The source-level `cargo check --tests` gate above passed and is the current evidence for this slice.
