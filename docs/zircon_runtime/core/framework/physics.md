---
related_code:
  - zircon_runtime/src/core/framework/physics/mod.rs
  - zircon_runtime/src/core/framework/physics/backend_state.rs
  - zircon_runtime/src/core/framework/physics/backend_status.rs
  - zircon_runtime/src/core/framework/physics/body_sync_state.rs
  - zircon_runtime/src/core/framework/physics/body_type.rs
  - zircon_runtime/src/core/framework/physics/collider_shape.rs
  - zircon_runtime/src/core/framework/physics/collider_sync_state.rs
  - zircon_runtime/src/core/framework/physics/combine_rule.rs
  - zircon_runtime/src/core/framework/physics/contact_event.rs
  - zircon_runtime/src/core/framework/physics/joint_constraint_metadata.rs
  - zircon_runtime/src/core/framework/physics/joint_drive.rs
  - zircon_runtime/src/core/framework/physics/joint_sync_state.rs
  - zircon_runtime/src/core/framework/physics/joint_type.rs
  - zircon_runtime/src/core/framework/physics/manager.rs
  - zircon_runtime/src/core/framework/physics/material_metadata.rs
  - zircon_runtime/src/core/framework/physics/material_sync_state.rs
  - zircon_runtime/src/core/framework/physics/query_filter.rs
  - zircon_runtime/src/core/framework/physics/ray_cast_hit.rs
  - zircon_runtime/src/core/framework/physics/ray_cast_query.rs
  - zircon_runtime/src/core/framework/physics/scene_step_result.rs
  - zircon_runtime/src/core/framework/physics/settings.rs
  - zircon_runtime/src/core/framework/physics/shape_cast_hit.rs
  - zircon_runtime/src/core/framework/physics/shape_cast_query.rs
  - zircon_runtime/src/core/framework/physics/shape_overlap_hit.rs
  - zircon_runtime/src/core/framework/physics/shape_overlap_query.rs
  - zircon_runtime/src/core/framework/physics/simulation_mode.rs
  - zircon_runtime/src/core/framework/physics/skeleton_joint_binding.rs
  - zircon_runtime/src/core/framework/physics/tests.rs
  - zircon_runtime/src/core/framework/physics/trigger_event.rs
  - zircon_runtime/src/core/framework/physics/trigger_event_kind.rs
  - zircon_runtime/src/core/framework/physics/world_step_plan.rs
  - zircon_runtime/src/core/framework/physics/world_sync_state.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/dynamic_scene/scene.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/augmentation/capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_classification/runtime/services.rs
  - zircon_plugins/physics/runtime/src/manager.rs
  - zircon_plugins/physics/runtime/src/query_contact.rs
  - zircon_plugins/physics/runtime/src/trigger.rs
  - zircon_plugins/physics/runtime/src/trigger/event.rs
  - zircon_plugins/physics/runtime/src/trigger/pair.rs
  - zircon_plugins/physics/runtime/src/trigger/point.rs
  - zircon_plugins/physics/runtime/src/trigger/scan.rs
  - zircon_plugins/physics/plugin.toml
implementation_files:
  - zircon_runtime/src/core/framework/physics/mod.rs
  - zircon_runtime/src/core/framework/physics/joint_constraint_metadata.rs
  - zircon_runtime/src/core/framework/physics/joint_drive.rs
  - zircon_runtime/src/core/framework/physics/joint_sync_state.rs
  - zircon_runtime/src/core/framework/physics/joint_type.rs
  - zircon_runtime/src/core/framework/physics/manager.rs
  - zircon_runtime/src/core/framework/physics/query_filter.rs
  - zircon_runtime/src/core/framework/physics/ray_cast_query.rs
  - zircon_runtime/src/core/framework/physics/shape_cast_hit.rs
  - zircon_runtime/src/core/framework/physics/shape_cast_query.rs
  - zircon_runtime/src/core/framework/physics/shape_overlap_hit.rs
  - zircon_runtime/src/core/framework/physics/shape_overlap_query.rs
  - zircon_runtime/src/core/framework/physics/skeleton_joint_binding.rs
  - zircon_runtime/src/core/framework/physics/trigger_event.rs
  - zircon_runtime/src/core/framework/physics/trigger_event_kind.rs
  - zircon_runtime/src/core/framework/physics/tests.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/dynamic_scene/scene.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/augmentation/capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_classification/runtime/services.rs
  - zircon_plugins/physics/plugin.toml
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/src/manager.rs
  - zircon_plugins/physics/runtime/src/trigger.rs
  - zircon_plugins/physics/runtime/src/trigger/event.rs
  - zircon_plugins/physics/runtime/src/trigger/pair.rs
  - zircon_plugins/physics/runtime/src/trigger/point.rs
  - zircon_plugins/physics/runtime/src/trigger/scan.rs
  - zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract/mod.rs
plan_sources:
  - user: 2026-06-04 plugin ecosystem infrastructure expansion
  - .codex/plans/ZirconEngine 周边设施与插件能力完善计划.md
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
tests:
  - zircon_runtime/src/core/framework/physics/tests.rs
  - backend_status_step_plan_and_physics_query_roundtrip_as_framework_dtos
  - shape_overlap_uses_shared_query_filter_for_layers_sensors_groups_and_exclusions
  - shape_overlap_rejects_non_finite_query_rotation
  - builtin_shape_cast_reports_initial_overlap_without_claiming_swept_solver
  - backend_status_step_plan_and_physics_query_roundtrip_as_framework_dtos includes Generic6Dof joint constraint and skeleton-binding serde round-trip coverage
  - world_project_roundtrip_preserves_physics_and_animation_components includes Generic6Dof joint metadata persistence through scene project IO
  - world_sync_preserves_constraint_and_skeletal_joint_metadata covers physics plugin world-sync projection into PhysicsJointSyncState
  - 2026-06-04: rustfmt --edition 2021 --check over runtime physics framework files, scene/project joint metadata files, physics plugin manager/query/trigger/lib/test files, scene asset fixtures, and manifest contribution tests (passed)
  - 2026-06-04: git diff --check over runtime physics framework tracked changes, scene/project joint metadata files, physics plugin files, and docs (passed with expected LF-to-CRLF warnings)
  - 2026-06-04: conflict-marker scan over touched Physics Rust/docs including untracked DTO/doc files (passed)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-physics-constraints-0604 --message-format short --color never (pending while active Cargo lanes are busy)
  - 2026-06-04: rustfmt --edition 2021 --check over physics trigger facade/child files and the Sound manager-handle structural test correction (passed)
  - 2026-06-04: git diff --check, trailing-whitespace scan, and conflict-marker scan over the trigger split, Physics/Sound docs, and manager-handle structural test correction (passed; expected LF-to-CRLF warnings only)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-physics-trigger-split-0604 --message-format short --color never (pending while active Cargo lanes are busy)
doc_type: module-detail
---

# Physics Framework Contracts

## Purpose

`zircon_runtime::core::framework::physics` is the neutral physics contract layer. It defines backend status, simulation settings, world sync DTOs, contact and trigger events, query DTOs, joint constraint metadata, skeleton joint bindings, and the `PhysicsManager` service trait that runtime plugins and scene hooks consume. It does not own a concrete physics backend, broadphase, solver, Jolt binding, editor panel, or persistent plugin state.

The current architecture keeps concrete behavior in `zircon_plugins/physics/runtime`. Runtime exposes stable contracts and manager resolver access; the plugin implements the manager and decides whether the active backend can step, sync, or answer queries.

## Related Files

The framework is folder-backed. `mod.rs` is only the public re-export surface.

- `settings.rs`, `backend_state.rs`, `backend_status.rs`, `simulation_mode.rs`, and `world_step_plan.rs` describe backend availability and fixed-step planning.
- `body_sync_state.rs`, `collider_sync_state.rs`, `joint_sync_state.rs`, `material_sync_state.rs`, and their enum/support files describe the scene-to-physics snapshot.
- `joint_constraint_metadata.rs`, `joint_drive.rs`, and `skeleton_joint_binding.rs` describe backend-neutral joint limits, drives, break/projection tolerances, and optional animation-skeleton/bone ownership metadata.
- `query_filter.rs`, `ray_cast_query.rs`, `shape_overlap_query.rs`, and `shape_cast_query.rs` define backend-neutral query inputs.
- `ray_cast_hit.rs`, `shape_overlap_hit.rs`, `shape_cast_hit.rs`, `contact_event.rs`, `trigger_event.rs`, and `trigger_event_kind.rs` define neutral query/contact/trigger outputs.
- `manager.rs` defines the `PhysicsManager` trait used by scene hooks and manager handles.
- `zircon_runtime/src/scene/components/scene.rs`, `zircon_runtime/src/asset/assets/scene.rs`, `zircon_runtime/src/scene/world/project_io.rs`, and `zircon_runtime/src/scene/dynamic_scene/scene.rs` persist and remap joint metadata without a plugin-local scene format.
- `tests.rs` locks default settings, serde shape, sync DTOs, and query DTO round-trips.

## Behavior Model

World sync is snapshot based. Runtime scene code or the physics plugin converts scene rigid-body, collider, joint, and material state into a `PhysicsWorldSyncState`. The concrete manager can sanitize that snapshot, cache it by `WorldHandle`, and answer queries without exposing backend objects.

Queries use a shared `PhysicsQueryFilter` instead of duplicating collision fields on each query type. The filter carries:

- optional `collision_mask`;
- `include_sensors`;
- `excluded_entities`;
- optional `required_collision_group`.

Ray casts carry origin, direction, max distance, and the shared filter. Shape overlap queries carry a collider shape plus world transform and filter. Shape casts carry a collider shape, origin transform, direction, max distance, and filter. The trait supplies default no-op `shape_overlap` and `shape_cast` implementations so simple managers can implement only stepping, sync, and ray casts while still satisfying the framework contract.

Joint sync now covers the basic family names needed by first-party authoring and runtime plugins: `Fixed`, `Distance`, `Hinge`, `Slider`, `ConeTwist`, and `Generic6Dof`. `PhysicsJointConstraintMetadata` is attached to both scene `JointComponent` and `PhysicsJointSyncState` so linear/angular limit metadata, per-axis drives, break thresholds, and projection tolerances can round-trip through ECS, scene assets, dynamic-scene remapping, and plugin world sync. `PhysicsSkeletonJointBinding` optionally links a joint to a skeleton entity and bone path for ragdoll/physical-bone style authoring. This is metadata and synchronization scope only; it is not a native constraint solver promise.

## Design and Rationale

The contract follows the engine boundary rule: `zircon_runtime::core::framework` defines stable data and traits, while plugin crates own backend-specific behavior. This avoids reintroducing a concrete `zircon_runtime::physics` implementation while still giving editor, scene, plugin, and scripting code one shared vocabulary for physics service access.

The shared filter is the important query invariant. It prevents ray, overlap, and cast query semantics from drifting apart as plugins add more query families. First-party plugin implementations should route all query matching through the same filter helper or an equivalent backend-native filter projection.

Shape overlap, shape cast, trigger events, constraints, and skeletal joints are neutral service capabilities, not a promise that every backend has full swept-volume or articulated-body support. The fallback plugin currently answers overlap from synchronized collider geometry, treats shape cast as an immediate overlap probe, computes trigger enter/stay/exit from synchronized sensor pairs, and carries joint constraint/skeleton metadata through sync. A native backend can later replace those behaviors behind the same `PhysicsManager` methods and DTOs.

The fallback trigger implementation is plugin-owned and folder-backed. Framework types define only `PhysicsTriggerEvent` and `PhysicsTriggerEventKind`; `zircon_plugins/physics/runtime/src/trigger/{scan,pair,event,point}.rs` owns synchronized collider scanning, trigger pair identity, event DTO construction, and finite fallback point selection.

## Control Flow

`zircon_plugins/physics/runtime` registers the concrete module and manager. Scene hooks resolve `PhysicsManager`, call `tick_scene_world(...)`, and receive a `PhysicsSceneStepResult` containing the step plan plus drained contacts and triggers. Query callers resolve the same manager and pass neutral DTOs. The framework never stores query state or performs collision tests itself.

The default `tick_scene_world(...)` implementation in the trait only plans, drains contacts/triggers, and returns the result. The plugin overrides it to run builtin fixed-step integration, rebuild the sync snapshot, and drain contact/trigger data from plugin-owned state.

## Edge Cases and Constraints

All query and joint DTOs are data contracts. Backend implementations must still validate finite transforms, finite vectors, non-negative distances, finite ordered limits, non-negative drive/break/projection values, non-empty skeleton bone paths when a skeleton binding is present, supported shapes, backend availability, and synchronized-world presence. Invalid inputs should produce empty query results, `None`, or be omitted from sanitized sync snapshots, not panics. The framework does not require a backend to allocate spatial acceleration structures, allocate native constraints, or retain entity references beyond the synchronized snapshot.

The no-op trait defaults are intentional. They keep optional backends from inventing plugin-local compatibility traits while still making missing capability behavior explicit: an implementation that does not override shape queries returns no hits.

## Test Coverage

Framework tests currently cover default settings, serde casing for collider/body/joint enums, world sync defaults, backend status/step-plan/query DTO round-trips, Generic6Dof joint constraint/skeleton metadata round-trips, and scene identity on body/collider/contact DTOs. Scene tests cover project IO persistence for joint metadata, and plugin runtime tests cover shared-filter overlap behavior, the builtin shape-cast boundary that only reports initial overlaps until a native swept solver is installed, and preservation of constraint/skeletal joint metadata in `build_world_sync_state`.

Focused Cargo validation for the current constraint-contract update is pending until active Cargo lanes quiet down. The next low-risk focused check is:

```powershell
cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-physics-constraints-0604 --message-format short --color never
```
