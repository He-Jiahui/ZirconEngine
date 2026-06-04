---
related_code:
  - zircon_plugins/ai/plugin.toml
  - zircon_plugins/ai/runtime/Cargo.toml
  - zircon_plugins/ai/runtime/src/lib.rs
  - zircon_plugins/ai/runtime/src/manager.rs
  - zircon_plugins/ai/runtime/src/manager/behavior_tree.rs
  - zircon_plugins/ai/runtime/src/manager/blackboard.rs
  - zircon_plugins/ai/runtime/src/manager/perception.rs
  - zircon_plugins/ai/runtime/src/manager/service.rs
  - zircon_plugins/ai/runtime/src/manager/snapshot.rs
  - zircon_plugins/ai/runtime/src/manager/state.rs
  - zircon_plugins/ai/runtime/src/manager/tick.rs
  - zircon_plugins/ai/runtime/src/manager/validation.rs
  - zircon_plugins/ai/runtime/src/module.rs
  - zircon_plugins/ai/runtime/src/tests/mod.rs
  - zircon_plugins/ai/runtime/src/tests/manager_validation.rs
  - zircon_plugins/ai/runtime/src/tests/module.rs
  - zircon_plugins/ai/runtime/src/tests/registration.rs
  - zircon_plugins/ai/runtime/src/tests/tick.rs
  - zircon_runtime/src/core/framework/ai/mod.rs
  - zircon_runtime/src/core/framework/ai/manager.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/augmentation/capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_classification/runtime/systems.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_rows/runtime/systems.rs
implementation_files:
  - zircon_plugins/ai/plugin.toml
  - zircon_plugins/ai/runtime/src/lib.rs
  - zircon_plugins/ai/runtime/src/manager.rs
  - zircon_plugins/ai/runtime/src/manager/behavior_tree.rs
  - zircon_plugins/ai/runtime/src/manager/blackboard.rs
  - zircon_plugins/ai/runtime/src/manager/perception.rs
  - zircon_plugins/ai/runtime/src/manager/service.rs
  - zircon_plugins/ai/runtime/src/manager/snapshot.rs
  - zircon_plugins/ai/runtime/src/manager/state.rs
  - zircon_plugins/ai/runtime/src/manager/tick.rs
  - zircon_plugins/ai/runtime/src/manager/validation.rs
  - zircon_plugins/ai/runtime/src/module.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/augmentation/capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_classification/runtime/systems.rs
tests:
  - zircon_plugins/ai/runtime/src/tests/mod.rs
  - zircon_plugins/ai/runtime/src/tests/manager_validation.rs
  - zircon_plugins/ai/runtime/src/tests/module.rs
  - zircon_plugins/ai/runtime/src/tests/registration.rs
  - zircon_plugins/ai/runtime/src/tests/tick.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_ai_runtime --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib runtime_experimental_plugin_toml_matches_catalog_partial_metadata --locked --jobs 1 --message-format short --color never
plan_sources:
  - user: 2026-06-04 plugin ecosystem infrastructure expansion
  - .codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md
  - .codex/plans/ZirconEngine 周边设施与插件能力完善计划.md
doc_type: module-detail
---

# AI Runtime Plugin

## Purpose

`zircon_plugins/ai/runtime` is the first-party optional AI runtime plugin. It contributes `AiModule`, exposes `DefaultAiManager` through the stable `AiManagerHandle`, and publishes package metadata for behavior-tree, blackboard, and perception capabilities. The plugin is experimental and partial: it establishes the reusable manager and data validation foundation, while full behavior-tree execution remains a later capability promotion.

## Related Files

`src/lib.rs` owns the runtime plugin descriptor, package manifest helpers, capability list, and extension registration. `src/module.rs` contributes the core module descriptor with an immediate driver, a lazy concrete manager, and a lazy neutral `AiManagerHandle`. `src/manager.rs` is now a structural manager entry that exposes `DefaultAiManager` and wires folder-backed manager responsibilities. `manager/state.rs` owns registered descriptors, blackboards, perceptions, active tree names, and last reports. `manager/service.rs` keeps the `AiManager` trait implementation as a delegation layer. `manager/behavior_tree.rs`, `blackboard.rs`, `perception.rs`, `tick.rs`, `snapshot.rs`, and `validation.rs` own their respective runtime operations. `src/tests/mod.rs` is a structural test entry point; its child modules verify registration/catalog parity, module resolution, manager contract validation, and tick/perception behavior.

## Behavior Model

The plugin registers these capabilities:

- `runtime.plugin.ai`
- `runtime.feature.ai.behavior_tree`
- `runtime.feature.ai.blackboard`
- `runtime.feature.ai.perception`

All four capabilities are currently `Partial`. This lets project/profile/export tooling present AI as an optional infrastructure plugin without pretending that full game AI execution is complete.

`DefaultAiManager` stores registered behavior-tree descriptors and blackboard schema descriptors behind stable numeric handles. It stores blackboard entries and perception snapshots per `(WorldHandle, EntityId)` and produces `AiRuntimeSnapshot` records for diagnostics or editor tooling.

## Control Flow

At registration time, `AiRuntimePlugin::register_runtime_extensions` calls `RuntimeExtensionRegistry::register_module(module_descriptor())`. When the module activates, `AiDriver` starts immediately. `DefaultAiManager` is lazy, and the public `AiManager` service resolves through `AiManagerHandle`, preserving the runtime manager boundary used by the rest of the engine.

At tick time, `manager/tick.rs` validates the requested behavior-tree handle, blackboard schema handle, finite delta seconds, blackboard entry keys and values, and optional perception snapshot before mutating runtime state. When a registered tree is present, the current staged implementation reports `Blocked` with the root node and an explicit diagnostic. Without a behavior tree, the agent is reported as `Idle`. Snapshot projection is isolated in `manager/snapshot.rs` so diagnostic/editor readers can grow without widening tick or validation logic.

## Edge Cases and Constraints

Behavior trees are rejected when ids are empty, node ids duplicate, the root node is missing, or a child edge points to an unknown node. Blackboard schemas are rejected when key names are empty, keys duplicate, or value type strings are unknown. Runtime blackboard input is rejected for duplicate entries, unknown schema keys, missing required keys, type mismatches, and non-finite scalar/vector values. Perception snapshots are rejected when the snapshot agent differs from the tick entity or when any stimulus contains non-finite position, strength, or age values.

The implementation deliberately does not add concrete task dispatch, decorator expression evaluation, pathfinding integration, or AI authoring UI. Those features should consume this manager boundary rather than widening `zircon_runtime` or adding feature-specific branches in shared foundations.

## Test Coverage

The plugin runtime test tree covers registration, module resolution, descriptor/catalog parity, behavior-tree validation, blackboard schema validation, staged tick behavior, schema-bound tick rejection, perception mismatch rejection, successful perception storage, and snapshot projection. Tests are split by responsibility so future behavior-tree execution, blackboard, perception, or module registration cases can grow without re-creating a monolithic crate-level test file.

The runtime manifest contribution test keeps static `plugin.toml`, linked descriptor metadata, and the built-in catalog in sync. The planned scoped verification commands are listed in the document header and should be rerun during the milestone testing stage.
