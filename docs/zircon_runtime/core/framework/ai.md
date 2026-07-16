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
  - zircon_runtime/Cargo.toml
  - zircon_app/Cargo.toml
  - zircon_plugins/ai/runtime/Cargo.toml
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
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/core/manager/service_names.rs
  - zircon_runtime/Cargo.toml
  - zircon_app/Cargo.toml
  - zircon_plugins/ai/runtime/Cargo.toml
plan_sources:
  - user: 2026-06-04 plugin ecosystem infrastructure expansion
  - .codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md
  - .codex/plans/ZirconEngine 周边设施与插件能力完善计划.md
  - docs/plans/zircon_plugins/06-ai.md
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
tests:
  - zircon_plugins/ai/runtime/src/tests/mod.rs
  - zircon_plugins/ai/runtime/src/tests/perception_conditions.rs
  - zircon_plugins/ai/runtime/src/tests/perception_runtime.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_ai_runtime --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib runtime_experimental_plugin_toml_matches_catalog_partial_metadata --locked --jobs 1 --message-format short --color never
  - python -m unittest tools.tests.test_frameworks_03_contract_feature_boundary
  - cargo +nightly check -p zircon_runtime --lib --no-default-features --features ai-contracts --locked --offline --jobs 1
doc_type: module-detail
---

# AI Framework Contracts

## Purpose

`zircon_runtime::core::framework::ai` is the neutral contract layer for optional AI plugins. It defines behavior-tree descriptors, blackboard schema/value DTOs, perception snapshots, agent tick requests, runtime snapshots, stable ids, and `AiManager`. The module does not execute gameplay AI by itself and does not link any concrete AI backend; concrete behavior belongs in `zircon_plugins/ai/runtime` or future VM/native plugin providers.

## Feature Boundary

The contract is compiled only by the `ai-contracts` Cargo feature. Runtime and App client/editor presets include this feature to preserve their default contract surface, while `target-server` does not add it implicitly. A server build that selects an AI provider receives the contract through that provider's explicit dependency feature instead of widening every headless build.

`zircon_plugins/ai/runtime` requests `zircon_runtime/ai-contracts` directly. The framework root and `core::manager` gate only module declarations, re-exports, resolver generation, and the canonical service name; AI DTO and manager behavior contain no feature-specific branch. This keeps `--no-default-features --features ai-contracts` additive and prevents a compatibility alias or runtime fallback from hiding an undeclared dependency.

## Behavior Model

The behavior-tree surface is descriptor-oriented. A tree has a stable string id, a root node id, and a list of node descriptors. Node kinds cover selector, sequence, parallel, decorator, service, task, and subtree so editor tools and VM plugins can describe common engine AI graphs without depending on plugin-owned Rust objects. Node parameters are typed DTOs, not a shared expression language: concrete plugins may interpret keys such as task `result`, decorator `blackboard_key`, scalar/vector equality parameters, numeric threshold parameters, decorator inversion flags, perception sense/source/strength/age filters, parallel policy strings, or subtree `behavior_tree` target ids, but the framework only preserves their typed values and finite-value contract. The first runtime plugin validates staged aliases, built-in parameter owners, perception filter values, and subtree target ids before registration so invalid authored statuses, node-kind mismatches, or missing referenced subtrees cannot silently become default execution behavior, while still preserving unknown plugin-specific parameter keys as neutral descriptor data.

The blackboard surface has two layers. `AiBlackboardSchemaDescriptor` declares allowed keys, value types, and whether each key is required. `AiBlackboardValueType` normalizes accepted schema spellings such as `bool`, `integer`, `scalar`, `string`, `vec3`, and `entity`. `AiBlackboardEntry` carries runtime values and exposes the value type so implementations can reject mismatched or non-finite data before agent state is mutated.

`AiAgentTickRequest` is the ECS-facing input contract. It identifies the world and entity, optionally names a registered behavior tree and blackboard schema, carries frame delta seconds, blackboard entries, and an optional `AiPerceptionSnapshot`. A perception snapshot belongs to the tick entity and contains sense-tagged stimuli with source entity, position, strength, and age fields; concrete runtimes may use it directly or combine it with stored manager state. `AiHearingStimulusEvent` is the neutral sound/animation/gameplay bus contract: producers provide source, world position, strength, optional maximum range, current age, and an origin tag without importing a concrete AI plugin. Concrete runtimes own bounded ingestion, retry, receiver fan-out, aging, and forgetting; producer subsystems remain independent of the AI plugin. `AiAgentTickReport` returns the decision status, active node id, and a diagnostic string when a concrete plugin intentionally blocks or stages behavior.

## Design and Rationale

The contract follows the repository boundary rule: shared runtime framework owns neutral DTOs and traits; manager access lives under `zircon_runtime::core::manager`; the concrete implementation lives in a plugin crate. This mirrors Unreal's split between AI controller, behavior tree, blackboard, and perception surfaces while keeping Zircon's Rust-side integration closer to Bevy-style plugin/profile selection.

Errors are structured through `AiManagerError` instead of only diagnostic strings. This is important for editor inspectors, VM plugins, and network/devtools surfaces because they need to distinguish missing roots, missing child nodes, invalid node child counts, invalid behavior-tree topology, invalid subtree targets, unknown handles, invalid built-in node parameter owners, invalid built-in node parameters, missing blackboard keys, type mismatches, non-finite values, and perception/entity mismatches.

## Edge Cases and Constraints

The framework accepts only normalized blackboard value types and finite scalar/vector payloads. Runtime implementations must reject duplicate tree ids, duplicate node ids, missing root nodes, missing child references, invalid node child counts for the concrete runtime, invalid descriptor topology such as cycles, duplicate/shared child edges, root incoming edges, and unreachable nodes, duplicate behavior-node parameters, non-finite behavior-node parameter values, invalid built-in node parameter owners, invalid built-in node parameter types or values, invalid task result aliases, invalid subtree target ids, invalid perception filter values for the concrete runtime, invalid policy values for the concrete runtime, duplicate schema keys, unknown schema value types, duplicate runtime entries, undeclared schema entries, required keys that are absent, type mismatches, non-finite scalar/vector values, non-finite tick deltas, and perception snapshots whose agent does not match the tick entity.

The current contract is intentionally data-oriented. It exposes node kinds, typed node parameters, snapshots, and normalized hearing events, but concrete selector, sequence, decorator, task, parallel, service, subtree, scan, aging, occlusion, or latent task semantics belong to plugin runtimes such as `zircon_plugins/ai/runtime`. The first concrete plugin uses these typed parameters for blackboard existence, exact equality including Vec3, integer threshold, scalar threshold, decorator inversion, perception stimulus filters, task-result, parallel-policy checks, and stable-id subtree reuse, following Unreal's blackboard decorator operation pattern, perception stimulus fields, and `RunBehavior` subtree-asset shape without adding a shared parser. EQS-style queries and persisted AI graph assets remain follow-up plugin/editor work.

## Test Coverage

`zircon_plugins/ai/runtime/src/tests/mod.rs` validates the first concrete implementation against this contract. It covers module registration, manager resolution, descriptor/catalog parity, behavior-tree child-reference, child-count, and topology validation, built-in node-parameter owner/type/value validation, subtree target validation, blackboard schema validation, tick schema checking, perception mismatch rejection, deterministic behavior-tree reports, decorator existence, absence, inversion, scalar/vector equality, numeric blackboard comparisons, perception-driven decorators over current and stored snapshots, budgeted source/receiver scans, forgetting, optional physics fallback, hearing event conversion, subtree execution through registered behavior-tree ids, and runtime snapshot projection.

The runtime plugin manifest test `runtime_experimental_plugin_toml_matches_catalog_partial_metadata` checks that static `plugin.toml`, the built-in catalog descriptor, and capability status rows agree for `runtime.plugin.ai`, `runtime.feature.ai.behavior_tree`, `runtime.feature.ai.blackboard`, and `runtime.feature.ai.perception`.

Frameworks 03 feature validation also compiles the contract without an implementation (`--no-default-features --features ai-contracts`), compiles the optional diagnostic adapter combination, checks Server without AI, and checks the AI plugin's explicit contract dependency. All four WSL nightly locked/offline commands passed on 2026-07-10; detailed evidence is recorded in `tests/acceptance/frameworks-03-ai-contract-feature-boundary.md`.
