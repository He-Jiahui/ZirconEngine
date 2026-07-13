---
related_code:
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/runtime_hook/mod.rs
  - zircon_runtime/src/scene/runtime_hook/set.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/scene/ecs/system/native/scheduled_scene_step.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/register/scene_hook.rs
  - zircon_runtime/src/plugin/extension_registry/access/scene_hook.rs
  - zircon_runtime/src/animation/scene_hook.rs
  - zircon_runtime/src/script/vm/scene_hook.rs
implementation_files:
  - zircon_runtime/src/scene/runtime_hook/mod.rs
  - zircon_runtime/src/scene/runtime_hook/set.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/scene/ecs/system/native/scheduled_scene_step.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/register/scene_hook.rs
  - zircon_runtime/src/plugin/extension_registry/access/scene_hook.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/zircon_runtime/frameworks/05/failure-2026-07-13-core-contract-reverse-dependencies.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - tools/tests/test_frameworks_05_layer_direction.py::Frameworks05LayerDirectionTests::test_scene_runtime_hooks_are_owned_and_stored_by_scene
  - zircon_runtime/src/scene/tests/ecs_schedule.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/fixed_update.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/world_driver.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_scene_hooks.rs
  - zircon_runtime/src/scene/tests/ecs_scheduled_native_systems.rs::world_driver_reuses_tick_schedule_snapshots_for_stage_runs
  - managed Windows cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked (pending until the repository Cargo testing stage is available)
doc_type: module-detail
---

# Scene Runtime Hooks

## Purpose

Scene runtime hooks are executable scene scheduling contracts. Their descriptor, callback context, registration wrapper, canonical ordering, and stage-indexed dispatch cache therefore belong to the scene subsystem—not the plugin facade and not the core runtime kernel.

`scene/runtime_hook/mod.rs` owns `SceneRuntimeHook`, `SceneRuntimeHookContext`, `SceneRuntimeHookDescriptor`, and `SceneRuntimeHookRegistration`. The context intentionally names `CoreHandle` and concrete `LevelSystem` because hook execution is a scene orchestration boundary. `set.rs` owns the canonical flat set and cached `SceneRuntimeHookStagePlan` used during a tick.

## Registration and Execution

Plugins may contribute scene-owned registrations through `RuntimeExtensionRegistry`. The registry validates plugin ownership, rejects duplicate ids, and returns ordered contributions; it does not own the executable protocol. After the scene module is active, `scene::install_scene_runtime_hooks(...)` resolves `WorldDriver` and merges registrations into the driver-owned set.

`WorldDriver` snapshots the stage plan once per tick. `SceneScheduleRunner` interleaves internal systems, native systems, runtime systems, deferred flush points, and the current stage's hook slice in deterministic stage/order/id order. Hooks execute outside the World mutex so they may use `LevelSystem` accessors safely.

## Kernel Boundary

The old `plugin/scene_hook/` owner, plugin root re-export, `core/runtime/state/scene_runtime_hooks.rs`, `CoreRuntime::install_scene_runtime_hooks`, and CoreHandle hook query/snapshot APIs were deleted together. Core runtime keeps only data-only `RuntimeDevtoolsSceneHookSnapshot` rows written by `WorldDriver`; it does not retain executable callbacks or a scene/plugin registry.

This split removes all production `animation→plugin` and `scene→plugin` hook references and one of the kernel's plugin-facade references. It preserves registration validation and devtools visibility without an alias, compatibility module, duplicate owner, or facade wrapper.

## Validation State

The focused ownership guard was introduced red and passed after the hard cut. The production dependency audit moved from 2,133 references / 74 edges and 31 total handoff violations to 2,139 / 72 and 25: `animation→plugin` is 0, `scene→plugin` is 0, and `script→plugin` fell from 2 to 1. The full Frameworks05 guard remains red until the remaining render, World, lifecycle, extension-registry, manifest, and VM bridge boundaries reach zero.
