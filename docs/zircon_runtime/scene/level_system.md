---
related_code:
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/level_system/physics_runtime_enabled.rs
  - zircon_runtime/src/scene/level_system/physics_runtime_disabled.rs
  - zircon_runtime/src/scene/module/default_level_manager.rs
  - zircon_runtime/src/scene/module/mod.rs
  - zircon_runtime/src/scene/module/level_manager_lifecycle.rs
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/asset_manager.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/level_system_render_extract.rs
  - zircon_runtime/src/scene/tests/mod.rs
implementation_files:
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/level_system/physics_runtime_enabled.rs
  - zircon_runtime/src/scene/level_system/physics_runtime_disabled.rs
  - zircon_runtime/src/scene/module/default_level_manager.rs
  - zircon_runtime/src/scene/module/mod.rs
  - zircon_runtime/src/scene/module/level_manager_lifecycle.rs
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/asset_manager.rs
  - zircon_runtime/src/scene/tests/mod.rs
plan_sources:
  - user: 2026-06-12 runtime architecture implementation from docs/plans/zircon_runtime/runtime
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/scene/tests/mod.rs
  - zircon_runtime/src/dynamic_api/tests/session_profiles.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/open_project.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs
  - tools/tests/test_frameworks_03_contract_feature_boundary.py
  - tests/acceptance/frameworks-03-physics-contract-feature-boundary.md
doc_type: module-detail
---

# Scene Level System

## Purpose

`LevelSystem` is the runtime owner for one live scene world plus per-level runtime state. It wraps the active `World`, cached physics and animation outputs, script binding start state, level metadata, lifecycle state, and registered subsystem names.

`DefaultLevelManager` owns the map from `WorldHandle` to `LevelSystem` instances and creates levels with stable handles. Its fallible creation path uses checked atomic allocation, admits `u64::MAX` once, and then returns `CoreError::LevelHandleExhausted`; it never wraps to the reserved zero handle.

## Optional Physics State

Physics step plans, contacts, and triggers are simulation outputs rather than always-on scene schema. `level_system.rs` now selects `physics_runtime_enabled.rs` or `physics_runtime_disabled.rs` at module declaration time. The enabled adapter owns the optional contract imports and the physics-specific `LevelSystem` methods; the disabled adapter contributes only an empty runtime-state slot. This keeps the shared declaration independent of `physics-contracts` while preserving one `WorldRuntimeState` lifecycle.

The hard cut replaced the inline Physics fields with `PhysicsRuntimeState`, gated Physics-only behavior at the adapter declaration, and removed direct `core::framework::physics` imports from `level_system.rs`. The disabled adapter does not expose stub simulation methods, and no compatibility re-export exists.

## Ownership Boundary

`LevelSystem` owns live level state. Callers use `snapshot`, `replace`, `with_world`, `with_world_mut`, and runtime-state record/read methods instead of reaching into the internal `Mutex` fields.

Animation pose publication accepts only `AnimationPoseSnapshot`, whose ordered entity map stores sealed `Arc<AnimationPoseOutput>` rows. Full evaluation replaces the outer snapshot; partial evaluation shallow-clones the ordered map only when a supplied or removed entity actually changes, reuses every unchanged row handle, and publishes the exact changed/removed entity list to the physics projection. Frame/render/history consumers keep the sealed rows shared. The public single-entity inspection method may materialize one owned pose, but no production whole-map compatibility API remains.

`DefaultLevelManager` owns the level map. The lifecycle owner creates and resolves levels through `try_create_level`, `try_create_default_level`, their explicit infallible convenience wrappers, and `level`. Multi-level callback traversal and VM type synchronization sort cloned level snapshots by `WorldHandle` before acquiring world locks, providing one stable lock/traversal order independent of HashMap iteration.

Runtime project startup loads its default level from the already activated Asset service project.
`scene::load_level_asset` receives the abstract `AssetManager` service and uses
`current_project_snapshot` to clone the authoritative, already scanned `ProjectManager` under a
short read lock. The lock is released before `DefaultLevelManager` performs scene file I/O or level
creation, preventing callback re-entry from deadlocking the Asset manager. The Scene owner does not
reopen the project path, rescan assets, or downcast to `ProjectAssetManager`; the snapshot cost is
explicit and preserves the activated manifest/registry revision.

## Poison Handling

The level owner exposes private field-specific lock helpers for world, runtime state, metadata, lifecycle, and subsystem storage. Those helpers recover poisoned mutexes with the owned inner state instead of panicking. This keeps a panic during a plugin/runtime callback from permanently crashing later level reads on the same holder.

`DefaultLevelManager` exposes `lock_levels()` only inside the scene module so lifecycle code can use the same poison-safe behavior for the level map.

Test code may intentionally call `lock().unwrap()` to poison a mutex, but production code in these owners must not directly call `.lock().unwrap(`.

## Validation

The 2026-07-11 Frameworks 03 static suite passes 27/27, including `test_optional_physics_runtime_state_uses_declaration_adapters`. The current Runtime `physics` filter passes 35/35, including `physics_runtime_state_records_and_resets_with_the_level`; nightly `core-min + physics-contracts` passes independently in 12m39s.

`scene::tests::level_system_state_locks_use_poison_recovery_helpers` scans the production section of the scene level owner files and rejects direct `.lock().unwrap(` usage. `level_system.rs` also has module-local poison coverage for world, runtime state, metadata, lifecycle, and subsystem holders.

Runtime 15 M3 F2 lock poison recovery guard / `runtime_15_f2_lock_poison_recovery_guard_core_min_cargo_passed_full_sweep_pending` adds `structure_convention/lock_poison_policy.rs::runtime_15_f2_lock_poison_recovery_guard_covers_scene_and_eventbus` as a cross-owner regression guard for this scene holder and the runtime EventBus lock helpers. `code_review_findings/p0_robustness.rs::review_f2_scene_eventbus_locks_recover_after_poison` mirrors the review table status as `scene/EventBus poison-safe lock recovery complete` and locks the module-local `level_system_accessors_recover_poisoned_state_locks` behavior test.

2026-06-22 F2 validation: scoped rustfmt/check passed for the touched scene files and guards; production static scan found no direct `.lock().unwrap(` in `level_system.rs`, `default_level_manager.rs`, or `level_manager_lifecycle.rs`. Focused Cargo `level_system_recovers_world_lock_after_writer_panic` was attempted with target dir `D:\cargo-targets\zircon-runtime07-lock-poison-0622` and timed out after 1200 seconds during compilation, so no package-level Cargo pass is claimed for this slice.

2026-06-27 F2 status closure validation: the new review guard first failed on missing `runtime_15_f2_lock_poison_recovery_guard_core_min_cargo_passed_full_sweep_pending` docs/status anchors. After the status mirrors were synced, `level_system_accessors_recover_poisoned_state_locks`, `review_f2_scene_eventbus_locks_recover_after_poison`, `runtime_15_f2_lock_poison_recovery_guard_covers_scene_and_eventbus`, `runtime_15_code_review_findings_tests_are_folder_backed`, and `status_output_tables` passed under core-min focused Cargo with target dir `E:\cargo-targets\zircon-runtime-f2-review-status-0627`. Full Runtime 15 Cargo sweep remains pending.
