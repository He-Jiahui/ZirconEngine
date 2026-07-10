---
related_code:
  - zircon_plugins/ai/plugin.toml
  - zircon_plugins/ai/runtime/Cargo.toml
  - zircon_plugins/ai/runtime/src/lib.rs
  - zircon_plugins/ai/runtime/src/manager.rs
  - zircon_plugins/ai/runtime/src/manager/behavior_tree.rs
  - zircon_plugins/ai/runtime/src/manager/blackboard.rs
  - zircon_plugins/ai/runtime/src/manager/execution.rs
  - zircon_plugins/ai/runtime/src/manager/parameters.rs
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
  - zircon_plugins/ai/runtime/src/tests/perception_conditions.rs
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
  - zircon_plugins/ai/runtime/src/manager/execution.rs
  - zircon_plugins/ai/runtime/src/manager/parameters.rs
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
  - zircon_plugins/ai/runtime/src/tests/perception_conditions.rs
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

`zircon_plugins/ai/runtime` is the first-party optional AI runtime plugin. It embeds the canonical `ai.runtime` module descriptor, exposes `DefaultAiManager` through the stable `AiManagerHandle`, and publishes package metadata for behavior-tree, blackboard, and perception capabilities. The plugin is experimental and partial: it establishes the reusable manager, data validation foundation, and deterministic descriptor-driven execution for the first behavior-tree node families while full game AI authoring and latent task dispatch remain later capability promotions.

## Related Files

`src/lib.rs` owns the runtime plugin descriptor, package manifest helpers, capability list, and extension registration. `src/module.rs` contributes the core module descriptor with an immediate driver, a lazy concrete manager, and a lazy neutral `AiManagerHandle`. `src/manager.rs` is now a structural manager entry that exposes `DefaultAiManager` and wires folder-backed manager responsibilities. `manager/state.rs` owns registered descriptors, blackboards, perceptions, active tree names, and last reports. `manager/service.rs` keeps the `AiManager` trait implementation as a delegation layer. `manager/behavior_tree.rs`, `blackboard.rs`, `perception.rs`, `tick.rs`, `snapshot.rs`, `execution.rs`, and `validation.rs` own their respective runtime operations. `src/tests/mod.rs` is a structural test entry point; its child modules verify registration/catalog parity, module resolution, manager contract validation, perception-condition behavior, and tick/perception behavior.

## Behavior Model

The plugin registers these capabilities:

- `runtime.plugin.ai`
- `runtime.feature.ai.behavior_tree`
- `runtime.feature.ai.blackboard`
- `runtime.feature.ai.perception`

All four capabilities are currently `Partial`. This lets project/profile/export tooling present AI as an optional infrastructure plugin without pretending that full game AI execution, authoring, EQS-style querying, or navigation coupling is complete.

`DefaultAiManager` stores registered behavior-tree descriptors and blackboard schema descriptors behind stable numeric handles. It stores blackboard entries and perception snapshots per `(WorldHandle, EntityId)` and produces `AiRuntimeSnapshot` records for diagnostics or editor tooling.

## Control Flow

At registration time, `RuntimePluginRegistrationReport::from_plugin(...)` installs the descriptor embedded in `AiRuntimePlugin`; the plugin no longer has a parallel `register_module(...)` path. When the module activates, `AiDriver` starts immediately. `DefaultAiManager` is lazy, and the public `AiManager` service resolves through `AiManagerHandle`, preserving the runtime manager boundary used by the rest of the engine.

At tick time, `manager/tick.rs` validates the requested behavior-tree handle, blackboard schema handle, finite delta seconds, blackboard entry keys and values, and optional perception snapshot before mutating runtime state. It snapshots registered descriptors and any stored perception snapshot under a short manager lock, releases the lock while running the deterministic executor, and writes blackboards, perceptions, active-tree state, and the last report back afterward. When a registered tree is present, `manager/execution.rs` evaluates it deterministically from descriptors against an execution context that contains the request blackboard plus the current request perception snapshot or the previously stored agent perception snapshot. Selector nodes return the first non-failed child, sequence nodes stop at the first non-succeeded child, decorators can gate exactly one child through validated and optionally inverted blackboard/perception comparisons, task nodes report a validated string `result` value with `running` as the default, parallel nodes fold child results through `success_policy` and `failure_policy` string parameters, and subtree nodes enter another registered behavior tree by stable descriptor id through the `behavior_tree` string parameter. `manager/parameters.rs` owns the shared staged parser and built-in parameter key groups so registration validation and execution cannot drift. The staged runtime accepts task `result` only on task nodes, parallel policies only on parallel nodes, blackboard and perception condition parameters only on decorators, and subtree `behavior_tree` targets only on subtree nodes. Unknown plugin-specific parameter keys remain valid neutral descriptor data until a concrete plugin or authoring layer claims them. Valid task result strings are `idle`, `running`, `in_progress`, `inprogress`, `succeeded`, `success`, `succeed`, `failed`, `failure`, `fail`, and `blocked`. Parallel defaults are `success_policy = "all"` and `failure_policy = "any"`; either policy may be changed to `"any"` or `"all"`. Subtree targets must refer to an already registered tree and cannot target their containing descriptor id; this keeps cross-tree reuse one-directional in the staged runtime and avoids recursive descriptor entry. Service nodes still report `Blocked` with an explicit diagnostic because no latent service scheduler has landed yet. Without a behavior tree, the agent is reported as `Idle`. Snapshot projection is isolated in `manager/snapshot.rs` so diagnostic/editor readers can grow without widening tick or validation logic.

Decorator blackboard comparisons are intentionally parameter-driven. `blackboard_key` selects one runtime entry, `exists` checks presence or absence, `equals_bool` / `equals_string` / `equals_integer` / `equals_scalar` / `equals_vec3` / `equals_entity` compare exact typed values, and numeric threshold parameters `greater_than_integer`, `greater_or_equal_integer`, `less_than_integer`, `less_or_equal_integer`, `greater_than_scalar`, `greater_or_equal_scalar`, `less_than_scalar`, and `less_or_equal_scalar` compare integer or scalar blackboard attributes. Multiple value comparison parameters on the same decorator are combined with deterministic AND semantics. The optional bool `invert` parameter flips the final condition gate before child selection, but it does not invert the child task or subtree result. This follows Unreal's raw decorator condition plus inverse flag, vector blackboard equality, and arithmetic operation shape while keeping the concrete interpretation in the AI runtime plugin rather than adding an expression evaluator to the shared framework.

Decorator perception comparisons use the same deterministic condition gate. `perception_sense` filters snapshot stimuli by normalized sense (`sight`, `hearing`, `damage`, `touch`, or `custom`, with common aliases accepted at runtime), `perception_source` filters by source entity, `perception_min_strength` filters weak stimuli, `perception_max_age_seconds` filters stale stimuli, and `perception_exists = false` turns the filter into an absence check. Perception filters AND-compose with blackboard comparisons before `invert` is applied. This follows Unreal's `FAIStimulus` shape of sense, strength, location, and age plus active stimulus queries, while keeping Zircon's current implementation as a pure descriptor tick instead of adding a perception aging system or latent service scheduler.

## Edge Cases and Constraints

Behavior trees are rejected when ids are empty, node ids duplicate, the root node is missing, a child edge points to an unknown node, a node kind has an invalid child count, or the descriptor graph is not a root-owned tree. Topology validation walks from the root, rejects cycles before the recursive executor can see them, requires the root to have no incoming edge, requires every non-root node to have exactly one incoming edge, rejects duplicate or shared child edges, and rejects unreachable nodes. Selector, sequence, and parallel nodes may own child lists; decorators must own exactly one child; task, service, and subtree descriptors must be childless in this staged runtime, matching Unreal/Fyrox-style leaf and asset-reference boundaries instead of silently ignoring child edges. Behavior-node parameters are rejected when keys duplicate, values are non-finite, built-in parameter owners do not match their node kind, built-in parameter types do not match their contract, task result strings are outside the supported staged status aliases, subtree target ids are missing, blank, not previously registered, or self-targeting, blackboard comparison or inversion parameters omit `blackboard_key`, vector equality parameters are not `vec3`, numeric comparison parameters do not use their required integer or scalar type, perception sense strings are unknown, perception source is not an entity value, perception strength/age thresholds are not scalar or are negative, `invert` is not a bool, or a parallel policy value is not `all` or `any`. Blackboard schemas are rejected when key names are empty, keys duplicate, or value type strings are unknown. Runtime blackboard input is rejected for duplicate entries, unknown schema keys, missing required keys, type mismatches, and non-finite scalar/vector values. Perception snapshots are rejected when the snapshot agent differs from the tick entity or when any stimulus contains non-finite position, strength, or age values.

The implementation deliberately does not add concrete task dispatch, latent service scheduling, perception aging/aggregation, dynamic subtree retargeting, decorator expression language, pathfinding integration, or AI authoring UI. Those features should consume this manager boundary rather than widening `zircon_runtime` or adding feature-specific branches in shared foundations.

## Test Coverage

The plugin runtime test tree covers registration, module resolution, descriptor/catalog parity, behavior-tree validation, node child-count validation, behavior-tree topology validation, built-in node-parameter owner/type/value validation, subtree target validation, selector/sequence/decorator/task/parallel/subtree execution, blackboard existence and absence checks, inverted decorator condition gates, exact Vec3 blackboard equality, integer and scalar blackboard threshold comparisons, schema-bound tick rejection, perception mismatch rejection, successful perception storage, perception-driven decorators using current or stored snapshots, sense/source/strength/age/absence perception filters, and snapshot projection. Tests are split by responsibility so future behavior-tree execution, blackboard, perception, or module registration cases can grow without re-creating a monolithic crate-level test file.

The runtime manifest contribution test keeps static `plugin.toml`, linked descriptor metadata, and the built-in catalog in sync. The planned scoped verification commands are listed in the document header and should be rerun during the milestone testing stage.
