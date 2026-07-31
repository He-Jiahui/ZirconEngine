---
related_code:
  - zircon_runtime/src/core/framework/scene/mod.rs
  - zircon_runtime/src/core/framework/scene/component_type_descriptor/mod.rs
  - zircon_runtime/src/core/framework/scene/component_type_descriptor/component_property_descriptor.rs
  - zircon_runtime/src/core/framework/scene/component_type_descriptor/component_type_descriptor.rs
  - zircon_runtime/src/core/framework/scene/component_type_descriptor/constructors.rs
  - zircon_runtime/src/core/framework/scene/entity_path.rs
  - zircon_runtime/src/core/framework/scene/level_summary.rs
  - zircon_runtime/src/core/framework/scene/mobility.rs
  - zircon_runtime/src/core/framework/scene/module_identity.rs
  - zircon_runtime/src/core/framework/scene/property_value.rs
  - zircon_runtime/src/core/framework/scene/resource.rs
  - zircon_runtime/src/core/framework/scene/system_stage.rs
  - zircon_runtime/src/core/framework/scene/world_handle.rs
  - zircon_runtime/src/core/framework/animation/event.rs
  - zircon_runtime/src/core/framework/navigation/agent.rs
  - zircon_runtime/src/core/framework/physics/skeletal_pose.rs
  - zircon_runtime/src/scene/runtime_extension/mod.rs
  - zircon_runtime/src/scene/runtime_hook/mod.rs
  - zircon_runtime/src/scene/navigation.rs
  - zircon_runtime/src/scene/ecs/mod.rs
  - zircon_runtime/src/scene/ecs/resource/mod.rs
  - zircon_runtime/src/scene/ecs/resource/registry.rs
implementation_files:
  - zircon_runtime/src/core/framework/scene/mod.rs
  - zircon_runtime/src/core/framework/scene/module_identity.rs
  - zircon_runtime/src/core/framework/scene/component_type_descriptor/mod.rs
  - zircon_runtime/src/core/framework/scene/component_type_descriptor/component_property_descriptor.rs
  - zircon_runtime/src/core/framework/scene/component_type_descriptor/component_type_descriptor.rs
  - zircon_runtime/src/core/framework/scene/component_type_descriptor/constructors.rs
  - zircon_runtime/src/core/framework/scene/resource.rs
  - zircon_runtime/src/core/framework/scene/system_stage.rs
  - zircon_runtime/src/core/framework/animation/event.rs
  - zircon_runtime/src/core/framework/navigation/agent.rs
  - zircon_runtime/src/core/framework/physics/skeletal_pose.rs
  - zircon_runtime/src/scene/runtime_extension/mod.rs
  - zircon_runtime/src/scene/runtime_hook/mod.rs
  - zircon_runtime/src/scene/navigation.rs
  - zircon_runtime/src/scene/ecs/mod.rs
  - zircon_runtime/src/scene/ecs/resource/mod.rs
  - zircon_runtime/src/scene/ecs/resource/registry.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/zircon_runtime/frameworks/01/fixed-2026-07-13-core-contract-reverse-dependencies.md
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python tools/runtime_domain_dependency_audit.py --pretty --output .codex/tmp/frameworks05-core-reverse-deps-after-stage.json
  - python -m unittest tools.tests.test_frameworks_03_contract_feature_boundary tools.tests.test_runtime_domain_dependency_audit
  - python -m unittest tools.tests.test_frameworks_05_layer_direction (historical 19/19 passed)
  - tools.tests.test_frameworks_05_layer_direction.Frameworks05LayerDirectionTests.test_scene_module_identity_has_one_neutral_contract_owner
  - python -m unittest tools.tests.test_frameworks_05_layer_direction -v (24/24 passed)
  - managed Windows cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked (passed)
doc_type: module-detail
---

# Scene Framework Contracts

## Purpose

`zircon_runtime::core::framework::scene` is the neutral scene vocabulary shared by the runtime kernel, optional framework domains, plugins, and the concrete scene implementation. It owns stable entity and world identities, property paths and values, level summaries, mobility, resource eligibility, and schedule-stage declarations. It does not own `World`, ECS storage, scene serialization, system registration, schedule execution, hierarchy repair, render extraction, or editor state.

This split is required for the future `zr_contracts` / `zr_kernel` crate boundary: lower-layer animation, navigation, physics, and lifecycle contracts must not import `asset`, `graphics`, `scene`, `plugin`, or another facade merely to name an entity, eligible resource, or schedule stage.

## Contract Owners

- `mod.rs` is the folder-backed declaration and curated export surface. `EntityId` and `NodeId` are stable identifiers; `LevelManager` exchanges only `WorldHandle`, `LevelSummary`, strings, and `CoreError`.
- `module_identity.rs` uniquely owns `SCENE_MODULE_NAME`. Scene, graphics, script, UI, builtin assembly and tests consume the contract constant directly; the concrete scene root has no duplicate definition or compatibility export.
- `component_type_descriptor/` uniquely defines plugin-contributed dynamic component and editable-property schema DTOs. Plugin manifests consume these records, while scene reflection and World registries own validation/application behavior.
- `entity_path.rs` and `property_value.rs` define serialized scene addressing and data-only property values without importing concrete ECS storage.
- `world_handle.rs` and `level_summary.rs` define stable world access and inspection records without exposing a `World` reference.
- `resource.rs` uniquely defines `SceneResource: 'static + Send + Sync`. Concrete resource slots, descriptors, ticks, and typed access stay in `scene/ecs`.
- `system_stage.rs` uniquely defines the nine-stage order, fixed-loop subset, rank, and fixed-loop classification. Concrete system registries and execution stay in `scene/ecs` and `scene/module`.

The scene and plugin domains may consume these lower-layer declarations as part of their public vocabulary, but neither may define a duplicate owner. The old `scene/ecs/resource/marker.rs`, `scene/ecs/system_stage.rs`, and `plugin/component_type_descriptor/` owners were deleted in the hard cutover.

## Dependency Direction

Framework consumers now import the canonical declarations directly:

- animation events use `core::framework::scene::EntityId`;
- navigation debug capture and skeletal-pose feeds implement `SceneResource` without importing scene storage;
- core runtime extension and hook stage planning use `core::framework::scene::SystemStage`;
- graphics/script/UI module descriptors use `core::framework::scene::SCENE_MODULE_NAME` and never import the concrete scene root merely to declare a dependency;
- shader IDE records use `core::resource::ResourceLocator` rather than the asset-owned `AssetUri` projection.

Executable Scene behavior stays above the contracts layer: `WorldDriver` owns runtime-hook and runtime-extension plans, `SceneNavigationRuntimeHandle` owns navigation execution against a World, and the Physics runtime plugin resolves its concrete manager before stepping a World. Core framework manager traits contain only neutral requests, handles, settings, query DTOs, and synchronization snapshots. No alias, string dispatch, `Any` escape hatch, compatibility module, or permanent dependency allowlist replaces the removed concrete edges.

## Runtime Flow

`SystemStage::ORDER` is consumed by scene schedule planning and `WorldDriver`. `SystemStage::FIXED_LOOP` drives the repeated `FixedFirst`, `FixedUpdate`, and `FixedPostUpdate` steps drained from the runtime time plan. Scene-owned hook and extension plans use the neutral enum while the scene implementation remains the sole callback executor against its concrete `World`; CoreRuntime stores neither executable callbacks nor a World-facing extension registry.

`SceneResource` is only an eligibility contract. `ResourceRegistry` assigns dense `ResourceId` values, `ResourceStore` owns erased values and change ticks, and World-facing APIs enforce typed access. Moving the marker does not move storage or behavior into framework.

## Reference Alignment

The ownership split follows repository-local mature-engine evidence. Bevy keeps typed/stable asset identifiers and handles in lower shared modules while concrete asset implementations live elsewhere. Fyrox separates resource identity and manager vocabulary from concrete data owners. Godot likewise uses lower stable UID/RID-style identities so upper systems do not import one another merely for identity. Zircon applies the same direction to scene ids, handles, resource eligibility, and schedule-stage vocabulary while deliberately retaining its concrete ECS and scheduler in the scene domain.

## Validation State

The 2026-07-18 module-identity hard cut started with one production graphics→scene edge at the graphics module descriptor. The guard was RED while `module_identity.rs` was absent, then GREEN after the canonical owner and all consumers moved. The full Frameworks05 layer-direction suite passes 24/24, the fresh production-only scan reports 2,380 refs / 78 edges with graphics→scene=0, and concrete scene-root or module-alias `SCENE_MODULE_NAME` consumers are zero. The expanded 27-path independent review reports P0/P1/P2=0. Both canonical lockfiles now contain Text01's `sys-locale`, but current-source managed Cargo is not yet accepted; this document does not claim package acceptance.

The 2026-07-13 Frameworks05 production-only baseline started at 2,151 references / 77 edges with 18 lower-to-upper and 38 internal-to-facade references. After the complete hard cut, the current baseline is 2,290 / 72 and all tracked reverse-layer/facade-inbound directions are zero. Frameworks05 passes 19/19, Frameworks03 plus dependency-audit passes 41/41, managed Windows Runtime `core-min`/default checks pass, and the focused `core-min` module-deactivation behavior passes 2/2. The handoff is returned as fixed under Frameworks01.
