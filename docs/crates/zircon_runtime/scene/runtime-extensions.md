---
related_code:
  - zircon_runtime/src/scene/runtime_extension/mod.rs
  - zircon_runtime/src/scene/runtime_extension/error.rs
  - zircon_runtime/src/scene/runtime_extension/plan.rs
  - zircon_runtime/src/scene/runtime_extension/registration.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/module/mod.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_world.rs
  - zircon_plugins/plugin_sdk/src/test.rs
  - zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract/mod.rs
implementation_files:
  - zircon_runtime/src/scene/runtime_extension/mod.rs
  - zircon_runtime/src/scene/runtime_extension/error.rs
  - zircon_runtime/src/scene/runtime_extension/plan.rs
  - zircon_runtime/src/scene/runtime_extension/registration.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/module/mod.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_world.rs
  - zircon_plugins/plugin_sdk/src/test.rs
  - zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract/mod.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
tests:
  - tools/tests/test_frameworks_05_layer_direction.py
  - zircon_runtime/src/scene/runtime_extension/mod.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry/tests.rs
---

# Scene Runtime Extension Plans

World-executable extension storage belongs to Scene, not Core or the Plugin facade. `WorldRuntimeExtensionPlan` is a scene-owned list of type-erased registrations keyed by stable family/id pairs. A plan can contain component descriptors, resource initializers, typed event registrations, native scene systems, and runtime scene systems without exposing plugin registry types to Scene.

`RuntimeExtensionRegistry::world_runtime_extension_plan()` is the upper-layer projection adapter. It clones the validated registrations into scene-owned commands. Native and runtime systems share the `system:<id>` key namespace, so duplicate system ids remain rejected across both families. `WorldDriver::install_world_runtime_extension_plan` transactionally merges a contribution and preserves the previously installed plan on duplicate failure.

Default and loaded levels ask `WorldDriver` to apply its current plan while the new World is being prepared. Every World therefore receives fresh resources, events, and system instances. `CoreRuntime` and `CoreHandle` no longer store a plugin registry, accept `scene::World`, or expose `install_world_runtime_extensions`.

The plugin SDK test runtime and Physics integration project a registry to a plan and install it through `scene::install_world_runtime_extension_plan`. The old Core installation entry and `WorldRuntimeExtensionSet` owner were deleted without aliases.

## Validation

- `test_world_runtime_extensions_are_planned_and_stored_by_scene` rejects Core ownership and locks the Scene/Plugin projection boundary.
- `failed_merge_preserves_the_previous_plan` covers transactional duplicate rejection at the Scene plan layer.
- The final production dependency audit records both `core→plugin = 0` and `core→scene = 0`. Managed Windows Runtime `core-min`/default checks, App/Editor checks, and Navigation/Physics/Plugin SDK test-target checks passed; the Frameworks05 layer-direction suite passed 19/19.
