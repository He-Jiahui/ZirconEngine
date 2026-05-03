---
related_code:
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/src/module.rs
  - zircon_runtime/src/physics/mod.rs
  - zircon_runtime/src/physics/runtime/mod.rs
  - zircon_runtime/src/physics/runtime/query_contact.rs
  - zircon_runtime/src/core/manager/service_names.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/tests/common/mod.rs
implementation_files:
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/src/module.rs
  - zircon_runtime/src/physics/mod.rs
  - zircon_runtime/src/physics/runtime/mod.rs
plan_sources:
  - user: 2026-05-03 继续补独立插件缺口
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
tests:
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_runtime/src/tests/extensions/animation_physics_absorption.rs
  - zircon_runtime/src/tests/extensions/manager_handles.rs
  - zircon_runtime/tests/common/mod.rs
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime -p zircon_plugin_animation_runtime --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-independent-plugin-physics-animation-cutover --message-format short --color never
  - cargo check -p zircon_runtime --lib --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-physics-animation-cutover --message-format short --color never
doc_type: module-detail
---

# Physics Runtime Plugin

`zircon_plugins/physics/runtime` now owns the physics runtime module descriptor. Its `module.rs` declares `PhysicsModule`, the plugin-local `PhysicsDriver`, and the manager descriptors that create `DefaultPhysicsManager` plus the canonical `PhysicsManagerHandle`.

`zircon_runtime::physics` remains the current contract owner for the shared manager implementation, scene sync helpers, default fallback step integration, query/contact code, and `PHYSICS_SETTINGS_CONFIG_KEY`. This is intentionally narrower than the previous state: runtime no longer exports `physics::module_descriptor`, `PhysicsModule`, or `PhysicsDriver`.

## Runtime Boundary

- The plugin crate contributes the lifecycle module through `RuntimeExtensionRegistry::register_module(module_descriptor())`.
- The canonical service name for `PhysicsManagerHandle` still comes from `zircon_runtime::core::manager::PHYSICS_MANAGER_NAME`.
- The default manager type is still `zircon_runtime::physics::DefaultPhysicsManager` until the next migration slice moves the full manager/backend implementation out of runtime.
- Runtime tests that need a local module now use explicit test fixtures instead of depending on a production runtime module descriptor.

## Remaining Migration

The module descriptor and driver are plugin-owned, but the simulation fallback, world sync state builder, contact/query implementation, and manager state are still in `zircon_runtime::physics`. The final cutover should move that manager behavior into this plugin or behind a plugin-owned backend contract, leaving runtime with only neutral physics DTOs and scene-facing contract surfaces.

`cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime -p zircon_plugin_animation_runtime --lib ...` timed out during Windows test-binary build/link under concurrent Cargo load on 2026-05-03. The source-level `cargo check --tests` gate above passed and is the current evidence for this slice.
