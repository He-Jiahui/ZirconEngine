---
related_code:
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/module/default_level_manager.rs
  - zircon_runtime/src/scene/module/level_manager_lifecycle.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/level_system_render_extract.rs
  - zircon_runtime/src/scene/tests/mod.rs
implementation_files:
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/module/default_level_manager.rs
  - zircon_runtime/src/scene/module/level_manager_lifecycle.rs
  - zircon_runtime/src/scene/tests/mod.rs
plan_sources:
  - user: 2026-06-12 runtime architecture implementation from docs/plans/zircon_runtime/runtime
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/scene/tests/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs
doc_type: module-detail
---

# Scene Level System

## Purpose

`LevelSystem` is the runtime owner for one live scene world plus per-level runtime state. It wraps the active `World`, cached physics and animation outputs, script binding start state, level metadata, lifecycle state, and registered subsystem names.

`DefaultLevelManager` owns the map from `WorldHandle` to `LevelSystem` instances and creates levels with stable handles.

## Ownership Boundary

`LevelSystem` owns live level state. Callers use `snapshot`, `replace`, `with_world`, `with_world_mut`, and runtime-state record/read methods instead of reaching into the internal `Mutex` fields.

`DefaultLevelManager` owns the level map. The lifecycle owner creates and resolves levels through `create_level`, `create_default_level`, and `level`.

## Poison Handling

The level owner exposes private field-specific lock helpers for world, runtime state, metadata, lifecycle, and subsystem storage. Those helpers recover poisoned mutexes with the owned inner state instead of panicking. This keeps a panic during a plugin/runtime callback from permanently crashing later level reads on the same holder.

`DefaultLevelManager` exposes `lock_levels()` only inside the scene module so lifecycle code can use the same poison-safe behavior for the level map.

Test code may intentionally call `lock().unwrap()` to poison a mutex, but production code in these owners must not directly call `.lock().unwrap(`.

## Validation

`scene::tests::level_system_state_locks_use_poison_recovery_helpers` scans the production section of the scene level owner files and rejects direct `.lock().unwrap(` usage. `level_system.rs` also has module-local poison coverage for world, runtime state, metadata, lifecycle, and subsystem holders.

Runtime 15 M3 F2 lock poison recovery guard / `runtime_15_f2_lock_poison_recovery_guard_static_passed_cargo_deferred` adds `structure_convention/lock_poison_policy.rs::runtime_15_f2_lock_poison_recovery_guard_covers_scene_and_eventbus` as a cross-owner regression guard for this scene holder and the runtime EventBus lock helpers.

2026-06-22 F2 validation: scoped rustfmt/check passed for the touched scene files and guards; production static scan found no direct `.lock().unwrap(` in `level_system.rs`, `default_level_manager.rs`, or `level_manager_lifecycle.rs`. Focused Cargo `level_system_recovers_world_lock_after_writer_panic` was attempted with target dir `D:\cargo-targets\zircon-runtime07-lock-poison-0622` and timed out after 1200 seconds during compilation, so no package-level Cargo pass is claimed for this slice.
