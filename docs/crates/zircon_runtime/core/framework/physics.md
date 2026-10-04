---
related_code:
  - zircon_runtime/src/core/framework/physics/mod.rs
  - zircon_runtime/src/core/framework/physics/backend_state.rs
  - zircon_runtime/src/core/framework/physics/backend_status.rs
  - zircon_runtime/src/core/framework/physics/body_sync_state.rs
  - zircon_runtime/src/core/framework/physics/body_type.rs
  - zircon_runtime/src/core/framework/physics/collider_shape.rs
  - zircon_runtime/src/core/framework/physics/collider_sync_state.rs
  - zircon_runtime/src/core/framework/physics/contact_event.rs
  - zircon_runtime/src/core/framework/physics/joint_sync_state.rs
  - zircon_runtime/src/core/framework/physics/joint_type.rs
  - zircon_runtime/src/core/framework/physics/manager.rs
  - zircon_runtime/src/core/framework/physics/material_sync_state.rs
  - zircon_runtime/src/core/framework/physics/query_filter.rs
  - zircon_runtime/src/core/framework/physics/ray_cast_hit.rs
  - zircon_runtime/src/core/framework/physics/ray_cast_query.rs
  - zircon_runtime/src/core/framework/physics/scene_step_result.rs
  - zircon_runtime/src/core/framework/physics/settings.rs
  - zircon_runtime/src/core/framework/physics/settings_store_error.rs
  - zircon_runtime/src/core/framework/physics/shape_cast_hit.rs
  - zircon_runtime/src/core/framework/physics/shape_cast_query.rs
  - zircon_runtime/src/core/framework/physics/shape_overlap_hit.rs
  - zircon_runtime/src/core/framework/physics/shape_overlap_query.rs
  - zircon_runtime/src/core/framework/physics/simulation_mode.rs
  - zircon_runtime/src/core/framework/physics/skeletal_pose.rs
  - zircon_runtime/src/core/framework/scene/physics/mod.rs
  - zircon_runtime/src/core/framework/scene/physics/joint_constraint_metadata.rs
  - zircon_runtime/src/core/framework/scene/physics/joint_constraint_serde.rs
  - zircon_runtime/src/core/framework/scene/physics/joint_drive.rs
  - zircon_runtime/src/core/framework/scene/physics/material_metadata.rs
  - zircon_runtime/src/core/framework/scene/physics/skeleton_joint_binding.rs
  - zircon_runtime/src/core/framework/physics/tests.rs
  - zircon_runtime/src/core/framework/physics/trigger_event.rs
  - zircon_runtime/src/core/framework/physics/trigger_event_kind.rs
  - zircon_runtime/src/core/framework/physics/world_step_plan.rs
  - zircon_runtime/src/core/framework/physics/world_sync_state.rs
  - zircon_runtime/src/asset/assets/scene/mod.rs
  - zircon_runtime/src/scene/components/scene/physics.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/mod.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/augmentation/capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_classification/runtime/services.rs
  - zircon_plugins/physics/runtime/src/manager.rs
  - zircon_plugins/physics/runtime/src/manager/service.rs
  - zircon_plugins/physics/runtime/src/runtime_system.rs
  - zircon_plugins/physics/runtime/src/backend/builtin/query_contact.rs
  - zircon_plugins/physics/runtime/src/backend/builtin/trigger.rs
  - zircon_plugins/physics/runtime/src/backend/builtin/trigger/event.rs
  - zircon_plugins/physics/runtime/src/backend/builtin/trigger/pair.rs
  - zircon_plugins/physics/runtime/src/backend/builtin/trigger/point.rs
  - zircon_plugins/physics/runtime/src/backend/builtin/trigger/scan.rs
  - zircon_plugins/physics/plugin.toml
implementation_files:
  - zircon_runtime/src/core/framework/physics/mod.rs
  - zircon_runtime/src/core/framework/physics/joint_sync_state.rs
  - zircon_runtime/src/core/framework/physics/joint_type.rs
  - zircon_runtime/src/core/framework/physics/manager.rs
  - zircon_runtime/src/core/framework/physics/query_filter.rs
  - zircon_runtime/src/core/framework/physics/ray_cast_query.rs
  - zircon_runtime/src/core/framework/physics/shape_cast_hit.rs
  - zircon_runtime/src/core/framework/physics/shape_cast_query.rs
  - zircon_runtime/src/core/framework/physics/shape_overlap_hit.rs
  - zircon_runtime/src/core/framework/physics/shape_overlap_query.rs
  - zircon_runtime/src/core/framework/scene/physics/skeleton_joint_binding.rs
  - zircon_runtime/src/core/framework/physics/trigger_event.rs
  - zircon_runtime/src/core/framework/physics/trigger_event_kind.rs
  - zircon_runtime/src/core/framework/physics/tests.rs
  - zircon_runtime/src/asset/assets/scene/mod.rs
  - zircon_runtime/src/scene/components/scene/physics.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/mod.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/augmentation/capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_classification/runtime/services.rs
  - zircon_plugins/physics/plugin.toml
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/src/manager.rs
  - zircon_plugins/physics/runtime/src/manager/service.rs
  - zircon_plugins/physics/runtime/src/runtime_system.rs
  - zircon_plugins/physics/runtime/src/backend/builtin/trigger.rs
  - zircon_plugins/physics/runtime/src/backend/builtin/trigger/event.rs
  - zircon_plugins/physics/runtime/src/backend/builtin/trigger/pair.rs
  - zircon_plugins/physics/runtime/src/backend/builtin/trigger/point.rs
  - zircon_plugins/physics/runtime/src/backend/builtin/trigger/scan.rs
  - zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract/mod.rs
plan_sources:
  - docs/plans/zircon_plugins/03-physics.md
  - docs/plans/zircon_plugins/04-animation.md
  - user: 2026-06-04 plugin ecosystem infrastructure expansion
  - .codex/plans/ZirconEngine 周边设施与插件能力完善计划.md
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
tests:
  - zircon_runtime/src/core/framework/physics/tests.rs
  - physics_settings_store_errors_are_domain_owned_and_stable
  - tools/tests/test_frameworks_01_physics_settings_error_boundary.py
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/state_machine_interruption.rs::pose_targets_visible_to_physics_step
  - zircon_runtime/tests/runtime_plugin_world_extensions_contract.rs
  - backend_status_step_plan_and_physics_query_roundtrip_as_framework_dtos
  - shape_overlap_uses_shared_query_filter_for_layers_sensors_groups_and_exclusions
  - shape_overlap_rejects_non_finite_query_rotation
  - builtin_shape_cast_reports_initial_overlap_without_claiming_swept_solver
  - backend_status_step_plan_and_physics_query_roundtrip_as_framework_dtos includes Generic6Dof joint constraint and skeleton-binding serde round-trip coverage
  - joint_constraint_metadata_toml_roundtrips_sparse_axis_limits covers default TOML serialization, sparse x/y/z axis maps, and legacy three-slot axis arrays
  - world_project_roundtrip_preserves_physics_and_animation_components includes Generic6Dof joint metadata persistence through scene project IO
  - world_sync_preserves_constraint_and_skeletal_joint_metadata covers physics plugin world-sync projection into PhysicsJointSyncState
  - 2026-06-04: rustfmt --edition 2021 --check over runtime physics framework files, scene/project joint metadata files, physics plugin manager/query/trigger/lib/test files, scene asset fixtures, and manifest contribution tests (passed)
  - 2026-06-04: git diff --check over runtime physics framework tracked changes, scene/project joint metadata files, physics plugin files, and docs (passed with expected LF-to-CRLF warnings)
  - 2026-06-04: conflict-marker scan over touched Physics Rust/docs including untracked DTO/doc files (passed)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-physics-constraints-0604 --message-format short --color never (pending while active Cargo lanes are busy)
  - 2026-06-04: rustfmt --edition 2021 --check over physics trigger facade/child files and the Sound manager-handle structural test correction (passed)
  - 2026-06-04: git diff --check, trailing-whitespace scan, and conflict-marker scan over the trigger split, Physics/Sound docs, and manager-handle structural test correction (passed; expected LF-to-CRLF warnings only)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-physics-trigger-split-0604 --message-format short --color never (pending while active Cargo lanes are busy)
  - 2026-06-05: cargo test -p zircon_runtime --lib core::framework::physics::tests::joint_constraint_metadata_toml_roundtrips_sparse_axis_limits --locked --jobs 1 --target-dir D:\cargo-targets\zircon-asset-test-splits-0605 --message-format short --color never -- --test-threads=1 --nocapture (passed)
  - 2026-06-05: cargo test -p zircon_runtime --lib asset::tests::assets::scene --locked --jobs 1 --target-dir D:\cargo-targets\zircon-asset-test-splits-0605 --message-format short --color never -- --test-threads=1 --nocapture (passed)
  - 2026-07-13: managed Windows cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_runtime -p zircon_plugin_physics_runtime -p zircon_plugin_sdk --tests --locked (passed)
doc_type: module-detail
---

# Physics Framework Contracts

`PhysicsManager` is a neutral settings/query/synchronization contract and does not accept `scene::World`. Concrete World stepping is plugin-owned: the Physics runtime system resolves `DefaultPhysicsManager` and calls its concrete `tick_scene_world`, while the neutral manager remains available for settings, queries, synchronization snapshots, contacts, and triggers. This removes the former Core→Scene reverse dependency without duplicating a World adapter or weakening the typed physics DTOs.

## Purpose

`zircon_runtime::core::framework::physics` is the optional neutral simulation contract layer. It defines backend status, simulation settings, world sync DTOs, contact and trigger events, query DTOs, and the `PhysicsManager` service trait that runtime plugins and scene hooks consume. Persisted material, joint-drive, constraint, and skeleton-binding schema belongs to the always-on `core::framework::scene::physics` owner. This module does not own a concrete physics backend, broadphase, solver, Jolt binding, editor panel, or persistent plugin state.

The current architecture keeps concrete behavior in `zircon_plugins/physics/runtime`. Runtime exposes stable contracts and manager resolver access; the plugin implements the manager and decides whether the active backend can step, sync, or answer queries.

## Related Files

The framework is folder-backed. `mod.rs` is only the public re-export surface.

- `settings.rs`, `settings_store_error.rs`, `backend_state.rs`, `backend_status.rs`, `simulation_mode.rs`, and `world_step_plan.rs` describe backend availability, settings persistence failures, and fixed-step planning.
- `body_sync_state.rs`, `collider_sync_state.rs`, `joint_sync_state.rs`, `material_sync_state.rs`, and their enum/support files describe the scene-to-physics snapshot while referencing persisted data from `core::framework::scene::physics`.
- `core::framework::scene::physics` owns backend-neutral authored joint limits, drives, break/projection tolerances, material metadata, and optional animation-skeleton/bone bindings.
- `query_filter.rs`, `ray_cast_query.rs`, `shape_overlap_query.rs`, and `shape_cast_query.rs` define backend-neutral query inputs.
- `ray_cast_hit.rs`, `shape_overlap_hit.rs`, `shape_cast_hit.rs`, `contact_event.rs`, `trigger_event.rs`, and `trigger_event_kind.rs` define neutral query/contact/trigger outputs.
- `manager.rs` defines the `PhysicsManager` trait used by scene hooks and manager handles.
- `zircon_runtime/src/scene/components/scene/physics.rs`, `zircon_runtime/src/asset/assets/scene/physics.rs`, `zircon_runtime/src/scene/world/project_io/physics.rs`, and `zircon_runtime/src/scene/dynamic_scene/scene/mod.rs` persist and remap joint metadata without a plugin-local scene format.
- `tests.rs` locks default settings, serde shape, sync DTOs, and query DTO round-trips.

## Behavior Model

World sync is snapshot based. Runtime scene code or the physics plugin converts scene rigid-body, collider, joint, and material state into a `PhysicsWorldSyncState`. The concrete manager can sanitize that snapshot, cache it by `WorldHandle`, and answer queries without exposing backend objects.

Queries use a shared `PhysicsQueryFilter` instead of duplicating collision fields on each query type. The filter carries:

- optional `collision_mask`;
- `include_sensors`;
- `excluded_entities`;
- optional `required_collision_group`.

Ray casts carry origin, direction, max distance, and the shared filter. Shape overlap queries carry a collider shape plus world transform and filter. Shape casts carry a collider shape, origin transform, direction, max distance, and filter. The trait supplies default no-op `shape_overlap` and `shape_cast` implementations so simple managers can implement only stepping, sync, and ray casts while still satisfying the framework contract.

Joint sync covers the basic family names needed by first-party authoring and runtime plugins: `Fixed`, `Distance`, `Hinge`, `Slider`, `ConeTwist`, and `Generic6Dof`. `PhysicsJointConstraintMetadata` from the always-on scene schema is attached to both scene `JointComponent` and optional `PhysicsJointSyncState`, so linear/angular limits, per-axis drives, break thresholds, and projection tolerances round-trip through ECS, scene assets, dynamic-scene remapping, and plugin world sync without a second owner. TOML serialization behavior lives beside that scene declaration in `joint_constraint_serde.rs`. `PhysicsSkeletonJointBinding` likewise remains authored scene metadata; it is not a native constraint solver promise.

## Design and Rationale

The contract follows the engine boundary rule: `zircon_runtime::core::framework` defines stable data and traits, while plugin crates own backend-specific behavior. This avoids reintroducing a concrete `zircon_runtime::physics` implementation while still giving editor, scene, plugin, and scripting code one shared vocabulary for physics service access.

The shared filter is the important query invariant. It prevents ray, overlap, and cast query semantics from drifting apart as plugins add more query families. First-party plugin implementations should route all query matching through the same filter helper or an equivalent backend-native filter projection.

Shape overlap, shape cast, trigger events, constraints, and skeletal joints are neutral service capabilities, not a promise that every backend has full swept-volume or articulated-body support. The fallback plugin currently answers overlap from synchronized collider geometry, treats shape cast as an immediate overlap probe, computes trigger enter/stay/exit from synchronized sensor pairs, and carries joint constraint/skeleton metadata through sync. A native backend can later replace those behaviors behind the same `PhysicsManager` methods and DTOs.

`SkeletalPoseTargets` and `SimulatedPoseFeed` form the neutral animation/physics ragdoll bridge. Both map a scene entity to immutable bone rows containing a canonical bone name, local transform, and normalized weight. Physics owns registration of both world resources; Animation may publish targets without importing a backend, while a future articulated-body implementation will publish simulated rows after fixed-step synchronization. Animation publication distinguishes a full snapshot from an explicit changed/removed entity delta: a full update clears and rebuilds the target store, while a partial update replaces only changed rows and calls `SkeletalPoseTargets::remove` only for removed entities. The bridge deliberately calls the transform local-space because the current Animation pose output does not carry a skeleton parent table needed to derive world-space bones.

The fallback trigger implementation is plugin-owned and folder-backed. Framework types define only `PhysicsTriggerEvent` and `PhysicsTriggerEventKind`; `zircon_plugins/physics/runtime/src/trigger/{scan,pair,event,point}.rs` owns synchronized collider scanning, trigger pair identity, event DTO construction, and finite fallback point selection.

## Control Flow

`zircon_plugins/physics/runtime` registers the concrete module and manager. Scene hooks resolve `PhysicsManager`, call `tick_scene_world(...)`, and receive a `PhysicsSceneStepResult` containing the step plan plus drained contacts and triggers. Query callers resolve the same manager and pass neutral DTOs. The framework never stores query state or performs collision tests itself.

`PhysicsManager::store_settings(...)` returns the contract-owned `PhysicsSettingsStoreError` rather
than kernel `CoreError`. Read-only backends return `ReadOnlyBackend`; concrete persistence owners
project their storage failure into `Persistence` at the plugin boundary. No `From<CoreError>` shim or
framework-to-kernel error dependency remains on this method.

The default `tick_scene_world(...)` implementation in the trait only plans, drains contacts/triggers, and returns the result. The plugin overrides it to run builtin fixed-step integration, rebuild the sync snapshot, and drain contact/trigger data from plugin-owned state.

## Edge Cases and Constraints

All query and joint DTOs are data contracts. Backend implementations must still validate finite transforms, finite vectors, non-negative distances, finite ordered limits, non-negative drive/break/projection values, non-empty skeleton bone paths when a skeleton binding is present, supported shapes, backend availability, and synchronized-world presence. Invalid inputs should produce empty query results, `None`, or be omitted from sanitized sync snapshots, not panics. Axis-limit deserialization rejects unknown map keys and arrays longer than the three supported axes so invalid authored data cannot silently drift into a fourth axis. The framework does not require a backend to allocate spatial acceleration structures, allocate native constraints, or retain entity references beyond the synchronized snapshot.

The no-op trait defaults are intentional. They keep optional backends from inventing plugin-local compatibility traits while still making missing capability behavior explicit: an implementation that does not override shape queries returns no hits.

## Test Coverage

Framework tests currently cover default settings, serde casing for collider/body/joint enums, world sync defaults, backend status/step-plan/query DTO round-trips, Generic6Dof joint constraint/skeleton metadata round-trips, TOML-safe default and sparse-axis joint constraint serialization, legacy axis-array deserialization, and scene identity on body/collider/contact DTOs. Scene tests cover project IO persistence for joint metadata and the scene asset round-trip where a default joint constraint is embedded in authored scene TOML. Plugin runtime tests cover shared-filter overlap behavior, the builtin shape-cast boundary that only reports initial overlaps until a native swept solver is installed, and preservation of constraint/skeletal joint metadata in `build_world_sync_state`.

Focused runtime validation for the TOML-safe joint-constraint continuation passed:

```powershell
cargo test -p zircon_runtime --lib core::framework::physics::tests::joint_constraint_metadata_toml_roundtrips_sparse_axis_limits --locked --jobs 1 --target-dir D:\cargo-targets\zircon-asset-test-splits-0605 --message-format short --color never -- --test-threads=1 --nocapture
cargo test -p zircon_runtime --lib asset::tests::assets::scene --locked --jobs 1 --target-dir D:\cargo-targets\zircon-asset-test-splits-0605 --message-format short --color never -- --test-threads=1 --nocapture
```

The 2026-07-11 hard-cut validation is broader: Frameworks 03 static gates pass 27/27, the current Runtime `physics` filter passes 35/35, nightly `core-min + physics-contracts` passes in 12m39s with 52 existing warnings, and nightly `target-server` passes in 15m14s with 53 existing warnings. These checks prove the optional contract compiles independently, Server excludes it, and persisted scene schema remains available through its always-on owner. After adding the skeletal-pose resources, the Physics plugin default suite passes 43/43 and the Animation bridge/full suites pass 1/1 and 78/78 respectively; simulated-feed writeback remains outside this completed slice.
