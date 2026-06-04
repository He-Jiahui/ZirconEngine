---
related_code:
  - zircon_runtime/src/core/framework/ai/mod.rs
  - zircon_runtime/src/core/framework/ai/behavior_tree.rs
  - zircon_runtime/src/core/framework/ai/blackboard.rs
  - zircon_runtime/src/core/framework/ai/error.rs
  - zircon_runtime/src/core/framework/ai/ids.rs
  - zircon_runtime/src/core/framework/ai/manager.rs
  - zircon_runtime/src/core/framework/ai/perception.rs
  - zircon_runtime/src/core/framework/ai/snapshot.rs
  - zircon_runtime/src/core/framework/ai/tick.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/core/manager/service_names.rs
implementation_files:
  - zircon_runtime/src/core/framework/ai/mod.rs
  - zircon_runtime/src/core/framework/ai/behavior_tree.rs
  - zircon_runtime/src/core/framework/ai/blackboard.rs
  - zircon_runtime/src/core/framework/ai/error.rs
  - zircon_runtime/src/core/framework/ai/ids.rs
  - zircon_runtime/src/core/framework/ai/manager.rs
  - zircon_runtime/src/core/framework/ai/perception.rs
  - zircon_runtime/src/core/framework/ai/snapshot.rs
  - zircon_runtime/src/core/framework/ai/tick.rs
plan_sources:
  - user: 2026-06-04 plugin ecosystem infrastructure expansion
  - .codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md
  - .codex/plans/ZirconEngine 周边设施与插件能力完善计划.md
tests:
  - zircon_plugins/ai/runtime/src/tests.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_ai_runtime --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib runtime_experimental_plugin_toml_matches_catalog_partial_metadata --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# AI Framework Contracts

## Purpose

`zircon_runtime::core::framework::ai` is the neutral contract layer for optional AI plugins. It defines behavior-tree descriptors, blackboard schema/value DTOs, perception snapshots, agent tick requests, runtime snapshots, stable ids, and `AiManager`. The module does not execute gameplay AI by itself and does not link any concrete AI backend; concrete behavior belongs in `zircon_plugins/ai/runtime` or future VM/native plugin providers.

## Behavior Model

The behavior-tree surface is descriptor-oriented. A tree has a stable string id, a root node id, and a list of node descriptors. Node kinds cover selector, sequence, parallel, decorator, service, task, and subtree so editor tools and VM plugins can describe common engine AI graphs without depending on plugin-owned Rust objects.

The blackboard surface has two layers. `AiBlackboardSchemaDescriptor` declares allowed keys, value types, and whether each key is required. `AiBlackboardValueType` normalizes accepted schema spellings such as `bool`, `integer`, `scalar`, `string`, `vec3`, and `entity`. `AiBlackboardEntry` carries runtime values and exposes the value type so implementations can reject mismatched or non-finite data before agent state is mutated.

`AiAgentTickRequest` is the ECS-facing input contract. It identifies the world and entity, optionally names a registered behavior tree and blackboard schema, carries frame delta seconds, blackboard entries, and an optional `AiPerceptionSnapshot`. `AiAgentTickReport` returns the decision status, active node id, and a diagnostic string when a plugin has intentionally staged behavior.

## Design and Rationale

The contract follows the repository boundary rule: shared runtime framework owns neutral DTOs and traits; manager access lives under `zircon_runtime::core::manager`; the concrete implementation lives in a plugin crate. This mirrors Unreal's split between AI controller, behavior tree, blackboard, and perception surfaces while keeping Zircon's Rust-side integration closer to Bevy-style plugin/profile selection.

Errors are structured through `AiManagerError` instead of only diagnostic strings. This is important for editor inspectors, VM plugins, and network/devtools surfaces because they need to distinguish missing roots, missing child nodes, unknown handles, missing blackboard keys, type mismatches, non-finite values, and perception/entity mismatches.

## Edge Cases and Constraints

The framework accepts only normalized blackboard value types. Runtime implementations must reject duplicate tree ids, duplicate node ids, missing root nodes, missing child references, duplicate schema keys, unknown schema value types, duplicate runtime entries, undeclared schema entries, required keys that are absent, type mismatches, non-finite scalar/vector values, non-finite tick deltas, and perception snapshots whose agent does not match the tick entity.

The current contract is intentionally data-oriented. It does not define task execution semantics, decorator expression language, EQS-style queries, navigation coupling, or persisted AI graph assets yet. Those belong in follow-up AI plugin or editor-authoring milestones after the framework DTOs are stable.

## Test Coverage

`zircon_plugins/ai/runtime/src/tests.rs` validates the first concrete implementation against this contract. It covers module registration, manager resolution, descriptor/catalog parity, behavior-tree child validation, blackboard schema validation, tick schema checking, perception mismatch rejection, and runtime snapshot projection.

The runtime plugin manifest test `runtime_experimental_plugin_toml_matches_catalog_partial_metadata` checks that static `plugin.toml`, the built-in catalog descriptor, and capability status rows agree for `runtime.plugin.ai`, `runtime.feature.ai.behavior_tree`, `runtime.feature.ai.blackboard`, and `runtime.feature.ai.perception`.
