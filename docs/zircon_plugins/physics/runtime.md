---
related_code:
  - zircon_plugins/physics/plugin.toml
  - zircon_plugins/physics/runtime/src/backend.rs
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/src/module.rs
  - zircon_plugins/physics/runtime/src/plugin.rs
  - zircon_plugins/physics/runtime/src/manager.rs
  - zircon_plugins/physics/runtime/src/manager/builtin_step.rs
  - zircon_plugins/physics/runtime/src/manager/clock.rs
  - zircon_plugins/physics/runtime/src/manager/query.rs
  - zircon_plugins/physics/runtime/src/manager/service.rs
  - zircon_plugins/physics/runtime/src/manager/settings.rs
  - zircon_plugins/physics/runtime/src/manager/validation.rs
  - zircon_plugins/physics/runtime/src/manager/world_sync.rs
  - zircon_plugins/physics/runtime/src/query_contact.rs
  - zircon_plugins/physics/runtime/src/query_contact/contact.rs
  - zircon_plugins/physics/runtime/src/query_contact/filter.rs
  - zircon_plugins/physics/runtime/src/query_contact/geometry.rs
  - zircon_plugins/physics/runtime/src/query_contact/overlap.rs
  - zircon_plugins/physics/runtime/src/query_contact/overlap/distance.rs
  - zircon_plugins/physics/runtime/src/query_contact/overlap/pairwise.rs
  - zircon_plugins/physics/runtime/src/query_contact/overlap/proxies.rs
  - zircon_plugins/physics/runtime/src/query_contact/overlap/query.rs
  - zircon_plugins/physics/runtime/src/query_contact/raycast.rs
  - zircon_plugins/physics/runtime/src/query_contact/raycast/aabb.rs
  - zircon_plugins/physics/runtime/src/query_contact/raycast/capsule.rs
  - zircon_plugins/physics/runtime/src/query_contact/raycast/quadratic.rs
  - zircon_plugins/physics/runtime/src/query_contact/raycast/sphere.rs
  - zircon_plugins/physics/runtime/src/runtime_system.rs
  - zircon_plugins/physics/runtime/src/trigger.rs
  - zircon_plugins/physics/runtime/src/trigger/event.rs
  - zircon_plugins/physics/runtime/src/trigger/pair.rs
  - zircon_plugins/physics/runtime/src/trigger/point.rs
  - zircon_plugins/physics/runtime/src/trigger/scan.rs
  - zircon_plugins/physics/editor/Cargo.toml
  - zircon_plugins/physics/editor/src/plugin.rs
  - zircon_plugins/physics/editor/src/tests.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/core/framework/physics/joint_constraint_metadata.rs
  - zircon_runtime/src/core/framework/physics/joint_drive.rs
  - zircon_runtime/src/core/framework/physics/joint_sync_state.rs
  - zircon_runtime/src/core/framework/physics/joint_type.rs
  - zircon_runtime/src/core/framework/physics/manager.rs
  - zircon_runtime/src/core/framework/physics/query_interface.rs
  - zircon_runtime/src/core/framework/physics/query_filter.rs
  - zircon_runtime/src/core/framework/physics/ray_cast_hit.rs
  - zircon_runtime/src/core/framework/physics/ray_cast_query.rs
  - zircon_runtime/src/core/framework/physics/scene_step_result.rs
  - zircon_runtime/src/core/framework/physics/shape_cast_hit.rs
  - zircon_runtime/src/core/framework/physics/shape_cast_query.rs
  - zircon_runtime/src/core/framework/physics/shape_overlap_hit.rs
  - zircon_runtime/src/core/framework/physics/shape_overlap_query.rs
  - zircon_runtime/src/core/framework/physics/skeleton_joint_binding.rs
  - zircon_runtime/src/core/framework/physics/trigger_event.rs
  - zircon_runtime/src/core/framework/physics/trigger_event_kind.rs
  - zircon_runtime/src/core/framework/physics/world_step_plan.rs
  - zircon_runtime/src/core/manager/service_names.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/augmentation/capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_classification/runtime/services.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/dynamic_scene/scene.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/plugin/scene_hook/mod.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
implementation_files:
  - zircon_plugins/physics/plugin.toml
  - zircon_plugins/physics/runtime/src/backend.rs
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/src/module.rs
  - zircon_plugins/physics/runtime/src/manager.rs
  - zircon_plugins/physics/runtime/src/manager/builtin_step.rs
  - zircon_plugins/physics/runtime/src/manager/clock.rs
  - zircon_plugins/physics/runtime/src/manager/query.rs
  - zircon_plugins/physics/runtime/src/manager/service.rs
  - zircon_plugins/physics/runtime/src/manager/settings.rs
  - zircon_plugins/physics/runtime/src/manager/validation.rs
  - zircon_plugins/physics/runtime/src/manager/world_sync.rs
  - zircon_plugins/physics/runtime/src/query_contact.rs
  - zircon_plugins/physics/runtime/src/query_contact/contact.rs
  - zircon_plugins/physics/runtime/src/query_contact/filter.rs
  - zircon_plugins/physics/runtime/src/query_contact/geometry.rs
  - zircon_plugins/physics/runtime/src/query_contact/overlap.rs
  - zircon_plugins/physics/runtime/src/query_contact/overlap/distance.rs
  - zircon_plugins/physics/runtime/src/query_contact/overlap/pairwise.rs
  - zircon_plugins/physics/runtime/src/query_contact/overlap/proxies.rs
  - zircon_plugins/physics/runtime/src/query_contact/overlap/query.rs
  - zircon_plugins/physics/runtime/src/query_contact/raycast.rs
  - zircon_plugins/physics/runtime/src/query_contact/raycast/aabb.rs
  - zircon_plugins/physics/runtime/src/query_contact/raycast/capsule.rs
  - zircon_plugins/physics/runtime/src/query_contact/raycast/quadratic.rs
  - zircon_plugins/physics/runtime/src/query_contact/raycast/sphere.rs
  - zircon_plugins/physics/runtime/src/runtime_system.rs
  - zircon_plugins/physics/runtime/src/trigger.rs
  - zircon_plugins/physics/runtime/src/trigger/event.rs
  - zircon_plugins/physics/runtime/src/trigger/pair.rs
  - zircon_plugins/physics/runtime/src/trigger/point.rs
  - zircon_plugins/physics/runtime/src/trigger/scan.rs
  - zircon_plugins/physics/editor/Cargo.toml
  - zircon_plugins/physics/editor/src/plugin.rs
  - zircon_plugins/physics/editor/src/tests.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/core/framework/physics/joint_constraint_metadata.rs
  - zircon_runtime/src/core/framework/physics/joint_drive.rs
  - zircon_runtime/src/core/framework/physics/joint_sync_state.rs
  - zircon_runtime/src/core/framework/physics/joint_type.rs
  - zircon_runtime/src/core/framework/physics/manager.rs
  - zircon_runtime/src/core/framework/physics/query_filter.rs
  - zircon_runtime/src/core/framework/physics/ray_cast_query.rs
  - zircon_runtime/src/core/framework/physics/scene_step_result.rs
  - zircon_runtime/src/core/framework/physics/shape_cast_hit.rs
  - zircon_runtime/src/core/framework/physics/shape_cast_query.rs
  - zircon_runtime/src/core/framework/physics/shape_overlap_hit.rs
  - zircon_runtime/src/core/framework/physics/shape_overlap_query.rs
  - zircon_runtime/src/core/framework/physics/skeleton_joint_binding.rs
  - zircon_runtime/src/core/framework/physics/trigger_event.rs
  - zircon_runtime/src/core/framework/physics/trigger_event_kind.rs
  - zircon_runtime/src/core/framework/physics/world_step_plan.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/augmentation/capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_classification/runtime/services.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/dynamic_scene/scene.rs
  - zircon_runtime/src/scene/world/project_io.rs
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
  - world_sync_preserves_constraint_and_skeletal_joint_metadata
  - world_project_roundtrip_preserves_physics_and_animation_components
  - backend_status_step_plan_and_physics_query_roundtrip_as_framework_dtos
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - cargo test --manifest-path "zircon_plugins/Cargo.toml" -p zircon_plugin_physics_runtime --test physics_manager_runtime_contract contract::step::builtin_fixed_step_uses_live_world_records_before_node_cache_flush --locked --quiet -- --exact --nocapture
  - zircon_runtime/src/tests/extensions/animation_physics_absorption.rs
  - zircon_runtime/src/tests/extensions/manager_handles.rs
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked --test physics_manager_runtime_contract --target-dir target\codex-shared-a
  - cargo check --manifest-path zircon_plugins/Cargo.toml --locked --target-dir target\codex-shared-a
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --tests --locked --quiet (blocked: unrelated active scene world/ECS compile errors)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-plugin-checks --message-format short --color never (2026-06-12 plugin-architecture runtime-system migration: passed with existing warnings; zircon_plugins/Cargo.lock protected/restored)
  - cargo test -p zircon_runtime --locked --lib --target-dir target\codex-shared-a
  - 2026-05-31: cargo test --manifest-path .\zircon_plugins\physics\runtime\Cargo.toml physics_registration_contributes_runtime_module --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-authoring-runtime-metadata --color never --quiet (red before linked capability-status metadata, then passed with existing runtime warnings)
  - 2026-05-31: cargo test --manifest-path .\Cargo.toml -p zircon_runtime --lib runtime_experimental_plugin_toml_matches_catalog_partial_metadata --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-authoring-runtime-metadata --color never --quiet (passed with existing runtime warnings)
  - 2026-06-04: rustfmt --edition 2021 --check over runtime physics framework and physics plugin manager/query files (passed)
  - 2026-06-04: git diff --check -- zircon_runtime/src/core/framework/physics zircon_plugins/physics/runtime/src (passed with expected LF-to-CRLF warnings)
  - 2026-06-04: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-physics-contract-0604 --message-format short --color never (pending while active Cargo lanes are busy)
  - 2026-06-04: rustfmt --edition 2021 --check over runtime physics framework files, scene/project joint metadata files, physics plugin manager/query/trigger/lib/test files, scene asset fixtures, and manifest contribution tests (passed)
  - 2026-06-04: git diff --check over runtime physics framework tracked changes, scene/project joint metadata files, physics plugin files, and docs (passed with expected LF-to-CRLF warnings)
  - 2026-06-04: conflict-marker scan over touched Physics Rust/docs including untracked DTO/doc files (passed)
  - 2026-06-04: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-physics-constraints-0604 --message-format short --color never (pending while active Cargo lanes are busy)
  - 2026-06-04: rustfmt --edition 2021 --check over physics trigger facade/child files and the Sound manager-handle structural test correction (passed)
  - 2026-06-04: git diff --check, trailing-whitespace scan, and conflict-marker scan over the trigger split, Physics/Sound docs, and manager-handle structural test correction (passed; expected LF-to-CRLF warnings only)
  - 2026-06-04: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-physics-trigger-facade-current-0604 --message-format short --color never (passed; log .codex/tmp/physics_trigger_facade_current_check_20260604.log, exit 0)
  - 2026-06-04: rustfmt --edition 2021 --check over physics raycast facade and raycast/{aabb,capsule,quadratic,sphere}.rs (passed)
  - 2026-06-04: git diff --check over tracked physics raycast docs/session changes, plus explicit trailing-whitespace and conflict-marker scans over the raycast Rust files, docs, and session note (passed; expected LF-to-CRLF warning only)
  - 2026-06-04: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-physics-raycast-split-0604 --message-format short --color never (pending while active Cargo/rustc lanes are busy)
  - 2026-06-04: rustfmt --edition 2021 --check over physics overlap facade and overlap/{distance,pairwise,proxies,query}.rs (passed)
  - 2026-06-04: git diff --check over tracked physics overlap docs/session changes, plus explicit trailing-whitespace and conflict-marker scans over the overlap Rust files, docs, and session note (passed; expected LF-to-CRLF warning only)
  - 2026-06-04: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-physics-overlap-split-0604 --message-format short --color never (pending while active Cargo/rustc lanes are busy)
doc_type: module-detail
---

# Physics Runtime Plugin

`zircon_plugins/physics/runtime` owns the concrete physics runtime after the hard cutover. The crate provides the `PhysicsModule` descriptor, the plugin-local `PhysicsDriver`, the explicit backend selector, the `DefaultPhysicsManager` fallback/backend state, ray/query and contact helpers, and the runtime scene system that runs physics at `SystemStage::FixedUpdate`.

`zircon_runtime` no longer exports `zircon_runtime::physics` and does not depend on the plugin crate. Runtime keeps only neutral contracts under `zircon_runtime::core::framework::physics`, manager service names/resolvers under `zircon_runtime::core::manager`, scene ECS state, and the generic runtime scene-system scheduling protocol.

The current backend option decision is recorded in [Physics Plugin Options](../physics-plugin-options.md): builtin remains the only executable V1 backend, Jolt is the future native backend direction but remains unavailable until a real plugin-owned bridge is linked, and Rapier is not introduced on the primary path.

## Runtime Boundary

- The plugin contributes the lifecycle module through `RuntimePluginRegistrationBuilder::new(registry).module(PLUGIN_RUNTIME_MODULE_NAME, module_descriptor())`.
- The plugin contributes tick behavior through `RuntimePluginModuleRegistration::runtime_scene_system(...)` as `physics.step` in `SystemStage::FixedUpdate`, in set `physics.simulation`; the plugin runtime system owner no longer receives `PluginModuleId` or calls `RuntimeExtensionRegistry::register_runtime_scene_system(...)` directly.
- D8 runtime registration builder original evidence paths are locked by `review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder` and status `d8_runtime_registration_builder_original_paths_static_passed_cargo_deferred`.
- D10 animation/physics bridge call migration exports the runtime-owned `physics.query.v1` bridge interface from this plugin. `PhysicsRuntimePlugin::register(...)` creates a shared `DefaultPhysicsManager`, registers it as the module manager, and exports it as `Arc<dyn PhysicsQueryInterface>` through `RuntimePluginModuleRegistration::export_interface::<dyn PhysicsQueryInterface>(...)`; module activation binds the same manager back to `CoreHandle` so settings persistence remains on the original path. Guard `review_d10_animation_physics_tests_use_sdk_bridge_call` records status `d10_animation_physics_bridge_call_static_passed_cargo_deferred` and locks consumers to `WeakBridge<dyn PhysicsQueryInterface>`.
- D5 editor authoring macro consumer guard keeps the editor package on the SDK macro path: `zircon_plugins/physics/editor/src/plugin.rs` uses `zircon_plugin_sdk::authoring_plugin!` with `mirrors_runtime_manifest: zircon_plugin_physics_runtime::package_manifest()` and only keeps the Physics-specific extension registration body outside the macro. Status `d5_editor_authoring_macro_consumers_static_passed_cargo_deferred` is locked by `review_d5_editor_authoring_plugins_use_sdk_macro`.
- D9 editor/runtime mirror consumer guard keeps the editor package tied to this runtime package manifest through the SDK declaration projection: editor tests assert `mirrored_runtime_package_id()`, and the package manifest carries both `zircon_plugin_physics_runtime::PHYSICS_RUNTIME_CAPABILITY` and the Physics authoring capability. `tools/audit_plugin_structure.py --json` reports `editor_runtime_mirror_violations = 0` and `d9_editor_runtime_mirror_gate_status = editor-runtime-mirror-clean`; status `d9_editor_runtime_mirror_consumers_static_passed_cargo_deferred` is locked by `review_d9_editor_runtime_mirror_consumers_use_sdk_declaration`.
- Static `zircon_plugins/physics/plugin.toml`, the linked runtime package manifest, and `RuntimePluginDescriptor::builtin_catalog()` all classify Physics as category `runtime`, maturity `experimental`, with partial status rows for `runtime.plugin.physics`, `runtime.capability.physics.raycast`, `runtime.capability.physics.overlap`, `runtime.capability.physics.shape_cast`, `runtime.capability.physics.trigger_events`, `runtime.capability.physics.constraints`, and `runtime.capability.physics.skeletal_joints`. This keeps package/export metadata consistent without promoting Jolt, swept-shape, trigger, constraint-solver, or ragdoll parity.
- `PhysicsRuntimeSystem` resolves `PhysicsManagerHandle` through the runtime manager resolver and calls `PhysicsManager::tick_scene_world(...)`. If no manager is active, it still records the neutral fallback physics step plan on `LevelSystem`.
- `PhysicsManager::tick_scene_world(...)` is the scheduled `FixedUpdate` entrypoint. It treats `delta_seconds` from `WorldDriver` as one already-drained runtime fixed timestep and emits a one-step `PhysicsWorldStepPlan` when the backend can simulate. The frame-delta accumulator stays behind `plan_world_step(...)` for direct manager planning callers, so scheduled fixed systems do not reaccumulate substeps.
- `DefaultPhysicsManager` owns settings persistence, per-world accumulator state, sync snapshots, ray-cast fallback, shape-overlap/initial-shape-cast fallback, contact fallback, trigger event fallback, and joint metadata validation/sync.
- `manager.rs` is now the structural entry for the fallback manager state and public `PhysicsTickPlan` alias. Manager behavior is folder-backed: `manager/settings.rs` owns construction and settings persistence, `clock.rs` owns fixed-step accumulator planning, `builtin_step.rs` owns builtin rigid-body writeback, `world_sync.rs` owns ECS-to-framework snapshot projection and sync sanitization, `query.rs` owns manager-level ray/shape query dispatch, `service.rs` owns the `PhysicsManager` trait implementation, and `validation.rs` owns finite-value, collider, material, joint, skeleton-binding, and query-direction validation helpers.
- `query_contact.rs` is now the structural entry for fallback query/contact behavior. It exposes crate-local facade functions for `DefaultPhysicsManager` and trigger evaluation while keeping child-module helpers private to the query/contact subtree. `query_contact/contact.rs` computes contact events, `filter.rs` owns collision mask/group/sensor query filtering, `overlap.rs` dispatches discrete collider and shape-overlap fallback, `overlap/{query,proxies,pairwise,distance}.rs` own query traversal, finite shape proxy extraction, pairwise shape dispatch, and shared distance math, `raycast.rs` dispatches ray casts by collider shape, `raycast/{aabb,capsule,quadratic,sphere}.rs` own the shape-specific hit math, and `geometry.rs` owns shared finite geometry, scaling, distance, and hit-position helpers.
- `trigger.rs` is now the structural entry for fallback trigger-event behavior. It exposes the crate-local per-world pair-map type needed by `DefaultPhysicsManager`, while `pair.rs` keeps trigger/other entity identity fields private to the trigger subtree. `trigger/scan.rs` owns synchronized collider pair scanning and enter/stay/exit comparison, `event.rs` constructs neutral `PhysicsTriggerEvent` DTOs, and `point.rs` owns finite fallback contact-point approximation.
- `DefaultPhysicsManager::advance_clock(...)` now fills `PhysicsWorldStepPlan.interpolation_alpha` from the remaining fixed-step accumulator, clamped to `0.0..=1.0`.
- Builtin fixed-step integration and world sync enumerate `World::node_records()` instead of the deferred `World::nodes()` cache, so `FixedUpdate` observes bodies and colliders spawned or mutated before the next `PostUpdate` node-cache refresh.
- `backend.rs` maps `PhysicsSettings` into the plugin-local runtime backend state. Only explicit `backend = "builtin"` activates the builtin fallback; unavailable backends do not silently fall through to builtin behavior.
- `zircon_runtime::scene::WorldDriver` dispatches installed runtime scene systems by schedule stage and contains no physics-specific logic.

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
- `PhysicsQueryFilter` is now the shared query filter for ray, shape-overlap, and shape-cast queries. It carries collision mask, sensor inclusion, excluded entities, and required collision group.
- `PhysicsRayCastQuery` uses the shared filter instead of duplicating query fields. `PhysicsShapeOverlapQuery` and `PhysicsShapeCastQuery` expose the same filter semantics for broader query families.
- `PhysicsManager::shape_overlap(...)` and `PhysicsManager::shape_cast(...)` are optional manager capabilities with default empty/no-hit implementations. The builtin fallback manager answers shape overlap from its synchronized collider snapshot and uses immediate overlap as the first shape-cast behavior; native backends can replace that behind the same trait.
- `PhysicsQueryInterface` is the plugin-to-plugin bridge contract for ray, shape-overlap, and shape-cast queries. Its `PHYSICS_QUERY_INTERFACE_ID` is `physics.query.v1`, and linked plugins should consume it through `WeakBridge<dyn PhysicsQueryInterface>` instead of resolving `DefaultPhysicsManager`.
- The builtin fallback deliberately does not report swept shape hits yet. A non-overlapping cast returns `None` even when the sweep direction would later cross a collider, so future Jolt/Rapier-style backend work can add continuous shape casts without silently changing the current fallback contract.
- `PhysicsJointType`, scene `JointKind`, and scene-asset `SceneJointKindAsset` now share the same fixed/distance/hinge/slider/cone-twist/Generic6Dof vocabulary. The plugin maps those scene values into `PhysicsJointSyncState`.
- `PhysicsJointConstraintMetadata` and `PhysicsJointDrive` carry per-axis linear/angular limits, per-axis drives, break thresholds, and projection tolerances through scene ECS, scene assets, project IO, and the synchronized physics snapshot.
- `PhysicsSkeletonJointBinding` links a joint to a skeleton entity and bone path for physical-bone or ragdoll authoring metadata. Dynamic scene remapping updates the skeleton entity alongside the joint's connected entity.
- The builtin fallback validates finite ordered limits, non-negative drive/break/projection values, and non-empty bone paths before accepting a joint into synchronized state. It does not allocate native constraint rows, solve articulated chains, or simulate ragdolls yet.
- `PHYSICS_MANAGER_NAME` remains the stable service name consumed by runtime/editor callers.

The plugin can evolve Jolt or another backend behind `DefaultPhysicsManager` or a plugin-owned service without reintroducing `zircon_runtime::physics`.

## Validation Evidence

- Previous hard-cutover evidence: `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked --test physics_manager_runtime_contract --target-dir target\codex-shared-a` passed with 21 plugin contract tests before the backend selector seam was added.
- Previous hard-cutover evidence: `cargo check --manifest-path zircon_plugins/Cargo.toml --locked --target-dir target\codex-shared-a` passed for the independent plugin workspace with physics included but still outside the root workspace.
- Previous hard-cutover evidence: `cargo test -p zircon_runtime --locked --lib --target-dir target\codex-shared-a` passed with 767 runtime lib tests, validating scene hook dispatch, manager contracts, and hard-cutover structural assertions without depending on the plugin crate.
- Current plugin-architecture slice: `runtime_system.rs` replaces the old root `scene_hook.rs` entry, `plugin.toml` declares `system_sets = ["physics.simulation"]` and `system_anchors = ["physics.step"]`, and the runtime contract test installs world runtime extensions through `CoreRuntime::install_world_runtime_extensions(...)` instead of manual hook installation. `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short --color never` passes with existing warnings. `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-plugin-checks --message-format short --color never` also passes with existing warnings after migrating the tests to `RuntimeTimeAdvance` tick inputs; the plugin lockfile was protected/restored around the offline check.
- Current fixed-update runtime-system seam: `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_physics_runtime runtime_fixed_update_runs_one_physics_step_without_reaccumulating --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-plugin-checks --message-format short --color never -- --nocapture` passed, proving `physics.step` does not feed the runtime fixed delta back through the manager's frame-level accumulator when the runtime fixed timestep and `PhysicsSettings.fixed_hz` differ. `cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_animation_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-plugin-checks --message-format short --color never` also passes with existing warnings after this correction.
- Current backend selector seam: `rustfmt --edition 2021` passed for the touched physics runtime source and test files.
- Current backend selector seam: `cargo check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_physics_runtime --tests --locked --target-dir "target\codex-shared-a"` is blocked before physics test execution by unrelated active renderer code in `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs`, where the call to `render_compiled_scene(...)` supplies 10 arguments while the callee takes 8.
- Current interpolation-alpha seam: `cargo check --manifest-path "zircon_plugins/Cargo.toml" -p zircon_plugin_physics_runtime --tests --locked --quiet` is blocked before physics test execution by unrelated active scene world/ECS errors: `rebuild_fixed_component_presence_for_entity` visibility and missing `flush_pending_scene_systems_if_ready` call sites.
- Current live-world fixed-step seam: `cargo test --manifest-path "zircon_plugins/Cargo.toml" -p zircon_plugin_physics_runtime --test physics_manager_runtime_contract contract::step::builtin_fixed_step_uses_live_world_records_before_node_cache_flush --locked --quiet -- --exact --nocapture` passed after confirming the regression failed against the stale node-cache path.
- Current shape-query contract seam: static validation passed with scoped `rustfmt --check` and `git diff --check` after adding the shared query-filter DTOs, public shape query DTO re-exports, `PhysicsManager` shape-query methods, plugin fallback overlap/cast implementation, and non-finite query-transform rejection. Focused Cargo validation is pending until the active workspace/render Cargo lane clears.
- Current constraint/skeletal-joint contract seam: tests now cover Generic6Dof joint constraint/skeleton metadata framework serde, scene project IO persistence, plugin world-sync projection, and manifest/catalog partial capability status rows. Static validation and focused Cargo evidence are tracked in the session note for this slice.
- Current D10 animation/physics bridge call migration: `physics.query.v1` is exported from the physics runtime plugin through the SDK bridge helper, public physics runtime re-exports `PhysicsQueryInterface` / `PHYSICS_QUERY_INTERFACE_ID`, and the animation/physics contract test calls ray/overlap/shape-cast through `WeakBridge<dyn PhysicsQueryInterface>`. Static guard `review_d10_animation_physics_tests_use_sdk_bridge_call` records status `d10_animation_physics_bridge_call_static_passed_cargo_deferred`; Cargo remains deferred for this implementation slice.
- Current query/contact boundary split: `zircon_plugins/physics/runtime/src/query_contact.rs` is reduced to a structural facade over contact, filter, geometry, overlap, and raycast child modules. The split preserves the builtin fallback behavior while giving future collider filters, shape queries, raycasts, trigger contact checks, and native-backend parity work separate homes.
- Current overlap boundary split: `zircon_plugins/physics/runtime/src/query_contact/overlap.rs` is reduced to a structural dispatcher over `overlap/query.rs`, `proxies.rs`, `pairwise.rs`, and `distance.rs`. The split preserves box/sphere/capsule overlap fallback behavior while preventing query traversal, shape proxy extraction, pairwise shape matching, and shared distance math from accumulating in one implementation file.
- Current overlap boundary split validation: static formatting, tracked diff hygiene, explicit trailing-whitespace scan, conflict-marker scan, and line-count audit passed on 2026-06-04. Focused Cargo validation is pending because the latest process poll still showed active other-session Cargo/rustc lanes.
- Current raycast boundary split: `zircon_plugins/physics/runtime/src/query_contact/raycast.rs` is reduced to a structural dispatcher over `raycast/aabb.rs`, `capsule.rs`, `quadratic.rs`, and `sphere.rs`. The split preserves box/sphere/capsule fallback behavior while preventing new shape hit math or shared quadratic helpers from accumulating in the dispatcher.
- Current raycast boundary split validation: static formatting, tracked diff hygiene, explicit trailing-whitespace scan, conflict-marker scan, and line-count audit passed on 2026-06-04. Focused Cargo validation is pending because the 15:15 +08:00 process poll still showed active other-session Cargo/rustc lanes.
- Current query/contact facade correction: 2026-06-04 root workspace validation exposed that the facade tried to re-export private child helpers. The facade now provides explicit crate-local wrapper functions instead, preserving the boundary without widening child helper visibility.
- Current manager boundary split: `zircon_plugins/physics/runtime/src/manager.rs` is reduced to a structural facade over settings, clock, builtin-step integration, world sync, query dispatch, service trait implementation, and validation child modules. The split preserves the builtin fallback behavior while giving future rigid body, collider/trigger, constraint, skeleton/ragdoll, query, native-backend, and scene-sync work separate homes.
- Current trigger boundary split: `zircon_plugins/physics/runtime/src/trigger.rs` is reduced to a structural facade over trigger scan, pair identity, event construction, and fallback contact-point approximation child modules. The split preserves the builtin sensor-pair enter/stay/exit behavior while giving future trigger shapes, per-shape filtering, native overlap callbacks, and trigger diagnostics separate homes.
- Current trigger boundary split validation: static formatting, diff hygiene, trailing-whitespace scan, conflict-marker scan, and line-count audit passed on 2026-06-04. Focused Cargo validation also passed with `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-physics-trigger-facade-current-0604 --message-format short --color never`, which wrote `.codex/tmp/physics_trigger_facade_current_check_20260604.log`, exit 0.
- Current trigger facade correction: 2026-06-04 root workspace validation exposed that the facade re-exported the trigger pair map while the child alias was private to `trigger/pair.rs`. The alias is now crate-visible so sibling manager modules can store opaque trigger-pair state through the structural facade without widening pair-field mutation outside the trigger subtree.
