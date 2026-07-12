---
related_code:
  - zircon_plugins/ai/plugin.toml
  - zircon_plugins/Cargo.toml
  - zircon_plugins/Cargo.lock
  - zircon_plugins/ai/runtime/Cargo.toml
  - zircon_plugins/ai/runtime/src/lib.rs
  - zircon_plugins/ai/runtime/src/manager.rs
  - zircon_plugins/ai/runtime/src/manager/execution_gate.rs
  - zircon_plugins/ai/runtime/src/behavior_tree.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/catalog.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/compile.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor/condition.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/nodes/mod.rs
  - zircon_plugins/ai/runtime/src/manager/behavior_tree.rs
  - zircon_plugins/ai/runtime/src/manager/blackboard.rs
  - zircon_plugins/ai/runtime/src/manager/parameters.rs
  - zircon_plugins/ai/runtime/src/manager/perception.rs
  - zircon_plugins/ai/runtime/src/manager/service.rs
  - zircon_plugins/ai/runtime/src/manager/snapshot.rs
  - zircon_plugins/ai/runtime/src/manager/state.rs
  - zircon_plugins/ai/runtime/src/manager/tick.rs
  - zircon_plugins/ai/runtime/src/manager/validation.rs
  - zircon_plugins/ai/runtime/src/module.rs
  - zircon_plugins/ai/runtime/src/plugin.rs
  - zircon_plugins/ai/runtime/src/plugin/registration.rs
  - zircon_plugins/ai/runtime/src/tick_lod.rs
  - zircon_plugins/ai/runtime/src/tests/mod.rs
  - zircon_plugins/ai/runtime/src/tests/manager_validation.rs
  - zircon_plugins/ai/runtime/src/tests/behavior_tree_compile.rs
  - zircon_plugins/ai/runtime/src/tests/behavior_tree_execution.rs
  - zircon_plugins/ai/runtime/src/tests/behavior_tree_node_catalog.rs
  - zircon_plugins/ai/runtime/src/tests/module.rs
  - zircon_plugins/ai/runtime/src/tests/perception_conditions.rs
  - zircon_plugins/ai/runtime/src/tests/registration.rs
  - zircon_plugins/ai/runtime/src/tests/tick.rs
  - zircon_runtime/src/core/framework/ai/mod.rs
  - zircon_runtime/src/core/framework/ai/behavior_tree.rs
  - zircon_runtime/src/core/framework/ai/error.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry/owner_revocation.rs
  - zircon_runtime/src/plugin/extension_registry/typed_extension_point.rs
  - zircon_runtime/src/core/framework/ai/manager.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/augmentation/capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_classification/runtime/systems.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_rows/runtime/systems.rs
implementation_files:
  - zircon_plugins/ai/plugin.toml
  - zircon_plugins/ai/runtime/src/lib.rs
  - zircon_plugins/ai/runtime/src/manager.rs
  - zircon_plugins/ai/runtime/src/behavior_tree.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/catalog.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/compile.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/executor/condition.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/nodes/mod.rs
  - zircon_plugins/ai/runtime/src/manager/behavior_tree.rs
  - zircon_plugins/ai/runtime/src/manager/blackboard.rs
  - zircon_plugins/ai/runtime/src/manager/parameters.rs
  - zircon_plugins/ai/runtime/src/manager/perception.rs
  - zircon_plugins/ai/runtime/src/manager/service.rs
  - zircon_plugins/ai/runtime/src/manager/snapshot.rs
  - zircon_plugins/ai/runtime/src/manager/state.rs
  - zircon_plugins/ai/runtime/src/manager/tick.rs
  - zircon_plugins/ai/runtime/src/manager/validation.rs
  - zircon_plugins/ai/runtime/src/module.rs
  - zircon_plugins/ai/runtime/src/plugin.rs
  - zircon_plugins/ai/runtime/src/plugin/registration.rs
  - zircon_plugins/ai/runtime/src/tick_lod.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/augmentation/capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_classification/runtime/systems.rs
tests:
  - zircon_plugins/ai/runtime/src/tests/mod.rs
  - zircon_plugins/ai/runtime/src/tests/manager_validation.rs
  - zircon_plugins/ai/runtime/src/tests/behavior_tree_compile.rs
  - zircon_plugins/ai/runtime/src/tests/behavior_tree_execution.rs
  - zircon_plugins/ai/runtime/src/tests/behavior_tree_node_catalog.rs
  - zircon_plugins/ai/runtime/src/tests/module.rs
  - zircon_plugins/ai/runtime/src/tests/perception_conditions.rs
  - zircon_plugins/ai/runtime/src/tests/registration.rs
  - zircon_plugins/ai/runtime/src/tests/tick.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_ai_runtime --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib runtime_experimental_plugin_toml_matches_catalog_partial_metadata --locked --jobs 1 --message-format short --color never
plan_sources:
  - docs/plans/zircon_plugins/06-ai.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - user: 2026-06-04 plugin ecosystem infrastructure expansion
  - .codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md
  - .codex/plans/ZirconEngine 周边设施与插件能力完善计划.md
doc_type: module-detail
---

# AI Runtime Plugin

## Purpose

`zircon_plugins/ai/runtime` is the first-party optional AI runtime plugin. It embeds the canonical `ai.runtime` module descriptor, exposes `DefaultAiManager` through the stable `AiManagerHandle`, and publishes package metadata for behavior-tree, blackboard, and perception capabilities. Plugins 06 M1 supplies compiled behavior-tree assets, a typed node implementation catalog, the standard v1 node families, per-agent execution state, and a real `ai.behavior_tick` Update system with LOD throttling. The package remains Experimental / Partial because observer aborts, dense blackboard storage, concrete integration tasks, full perception, and editor tooling belong to M2–M5.

## Related Files

`src/behavior_tree.rs` is the domain entry. `behavior_tree/compile.rs` parses `.btree.toml` into the neutral descriptor and compiles a preorder dense node array plus a separate dense direct-child index table. `catalog.rs` owns the `TypedExtensionPoint`-backed implementation catalog and the typed `ai.behavior_node_registry.v1` runtime interface; `nodes/{composite,decorator,service,task}.rs` own the standard declarations. `executor.rs` owns traversal and per-agent state, while `executor/condition.rs` isolates blackboard/perception predicate evaluation. The old `manager/execution.rs` was deleted in the planned hard cutover. `manager/behavior_tree.rs` validates and compiles registrations against the manager's live shared catalog; `manager/tick.rs` keeps lock scope outside execution and persists instance state. `registration.rs` owns the extension-interface export, event catalog, and real Update-stage system registration, and `tick_lod.rs` owns deterministic Full/Half/Quarter scheduling.

## Behavior Model

The plugin registers these capabilities:

- `runtime.plugin.ai`
- `runtime.feature.ai.behavior_tree`
- `runtime.feature.ai.blackboard`
- `runtime.feature.ai.perception`

All four capabilities are currently `Partial`. This lets project/profile/export tooling present AI as an optional infrastructure plugin without pretending that full game AI execution, authoring, EQS-style querying, or navigation coupling is complete.

`DefaultAiManager` stores registered behavior-tree descriptors and blackboard schema descriptors behind stable numeric handles. It stores blackboard entries and perception snapshots per `(WorldHandle, EntityId)` and produces `AiRuntimeSnapshot` records for diagnostics or editor tooling.

## Control Flow

At registration time, `AiRuntimePlugin` constructs one shared `Arc<DefaultAiManager>` for the module service descriptor, runtime scene system, and behavior-node registry service. The plugin exports that service through `RuntimeExtensionRegistry` as the typed `ai.behavior_node_registry.v1` interface declared by both its linked descriptor and `plugin.toml`. Rust plugins can resolve the interface, contribute `BehaviorNodeDescriptor` values, and immediately register trees against the same live `RwLock`-backed catalog owned by the AI manager. Bootstrap standard-node rows are rebound in place to the actual interned AI owner, preserving their stable slots even when callers compiled trees through the public manager before plugin registration.

The central registry's owner-revocation listener first seals the owner against new execution leases, waits for in-flight node code and runtime objects, then removes that owner's private catalog slots and retires compiled trees, active instances, and reports before the contributor can unload. Next-generation registration waits for old cleanup and is linearized by the same owner lease. Composition code may also seed the catalog through `AiRuntimePlugin::with_behavior_node_catalog` or `DefaultAiManager::with_behavior_node_catalog`. An external node contributes a factory for a per-agent `BehaviorNodeRuntime` object whose tick context exposes parameters, blackboard, perception and elapsed delta. Standard nodes use the same catalog path. VM-backed factories remain M3-T3 behind the ZrVM host-handle dependency. Tree registration snapshots the current catalog, validates the versioned DTO, and compiles implementation names into stable dense catalog slots, semantics, optional factories, parameters, and direct-child ranges. Missing implementations and malformed topology return typed errors rather than flattened strings.

At tick time, `manager/tick.rs` snapshots the compiled tree set, schema and perception under a short poison-recovering lock, removes the agent instance state, executes without holding the manager lock, then writes the instance and report back. Switching the root tree clears that agent's prior node state. Sequence retains its active-child cursor, Parallel retains terminal branch results, and Selector resumes its Running child while re-evaluating higher-priority branches marked `RecheckWhileLowerPriorityRuns` and reusing cached terminal results for stable branches. BlackboardCondition, Cooldown, and RunSubtree use the reactive policy by default; external condition-like nodes opt in explicitly, while side-effecting external tasks remain stable. This prevents terminal sibling side effects from replaying without suppressing priority changes. RandomSelector uses non-negative `weight.<child_id>` or `weight_<index>` parameters and retains a selected Running branch. Cooldown, TimeLimit, Loop and Wait retain timers/counters per agent; Inverter and ForceResult transform terminal status; BlackboardCondition evaluates typed blackboard/perception predicates; UpdateBlackboardDistance exposes service callback status; RunSubtree enters a registered compiled tree with recursion protection. MoveTo, PlayAnimation, SetBlackboard, EmitEvent and ScriptTask have catalog identities and stable three-state placeholder dispatch; their real subsystem handoffs remain M3.

The registered `ai.behavior_tick` system runs at `SystemStage::Update`. It reads the active camera and agent world transforms, maps distance to Full/Half/Quarter LOD, deterministically staggers half/quarter agents, accumulates skipped elapsed time per agent so timed nodes retain wall-clock duration, ticks the same shared manager, and emits `AiAgentTickReport` through the scene event store. Missing transforms conservatively use Full rate.

Decorator blackboard comparisons are intentionally parameter-driven. `blackboard_key` selects one runtime entry, `exists` checks presence or absence, `equals_bool` / `equals_string` / `equals_integer` / `equals_scalar` / `equals_vec3` / `equals_entity` compare exact typed values, and numeric threshold parameters `greater_than_integer`, `greater_or_equal_integer`, `less_than_integer`, `less_or_equal_integer`, `greater_than_scalar`, `greater_or_equal_scalar`, `less_than_scalar`, and `less_or_equal_scalar` compare integer or scalar blackboard attributes. Multiple value comparison parameters on the same decorator are combined with deterministic AND semantics. The optional bool `invert` parameter flips the final condition gate before child selection, but it does not invert the child task or subtree result. This follows Unreal's raw decorator condition plus inverse flag, vector blackboard equality, and arithmetic operation shape while keeping the concrete interpretation in the AI runtime plugin rather than adding an expression evaluator to the shared framework.

Decorator perception comparisons use the same deterministic condition gate. `perception_sense` filters snapshot stimuli by normalized sense (`sight`, `hearing`, `damage`, `touch`, or `custom`, with common aliases accepted at runtime), `perception_source` filters by source entity, `perception_min_strength` filters weak stimuli, `perception_max_age_seconds` filters stale stimuli, and `perception_exists = false` turns the filter into an absence check. Perception filters AND-compose with blackboard comparisons before `invert` is applied. This follows Unreal's `FAIStimulus` shape of sense, strength, location, and age plus active stimulus queries, while keeping Zircon's current implementation as a pure descriptor tick instead of adding a perception aging system or latent service scheduler.

## Edge Cases and Constraints

Behavior trees are rejected when the explicit format version is not `1`, ids are empty, node ids duplicate, the root node is missing, a child edge points to an unknown node, the DTO kind disagrees with the implementation catalog category, a node kind has an invalid child count, or the descriptor graph is not a root-owned tree. `.btree.toml` parsing preserves the TOML source error and runs the same structural/parameter validation before dense compilation; registered subtree existence is deferred to manager registration, where the complete registered-tree set is available. Topology validation walks from the root, rejects cycles before the recursive executor can see them, requires the root to have no incoming edge, requires every non-root node to have exactly one incoming edge, rejects duplicate or shared child edges, and rejects unreachable nodes. Selector, sequence, and parallel nodes may own child lists; decorators must own exactly one child; task, service, and subtree descriptors must be childless in this staged runtime. Behavior-node parameters are rejected when keys duplicate, values are non-finite, built-in parameter owners/types/values violate their contracts, or timing/count/weight/result values are invalid. Blackboard schemas, runtime inputs, and perception snapshots retain their typed validation boundaries.

The implementation deliberately does not claim M2 observer aborts/dense blackboard storage, M3 subsystem task handoffs, M4 perception scanning/aging, or M5 editor/debug tooling. Those features consume the compiled tree, injected catalog, manager, event, and Update-system seams established by M1.

## Test Coverage

The M1 suite covers versioned `.btree.toml` loading and full validation, preorder/direct-child consistency, category mismatch, duplicate ownership and cycle rejection, the exact 18-entry standard catalog snapshot, dense stable slots, duplicate catalog ids, stateful external Rust factory injection, a live `RuntimeExtensionRegistry` contribution-and-owner-revocation path, actual nonzero AI owner binding, in-flight execution barriers and linearized next-generation registration, all standard node families across Success/Failure/Running including RunSubtree, composite Running resume without terminal sibling replay, explicit external and Cooldown selector rechecks, RandomSelector weight/stability, Wait/Cooldown/TimeLimit/Loop retained state, Quarter-LOD elapsed-time accumulation, Update-stage anchor registration, generated manifest event/system/interface parity, and deterministic LOD rates. Existing validation, blackboard, perception-condition, module and manifest tests remain green. The Windows acceptance run is `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_ai_runtime --locked --offline --jobs 1 --target-dir E:\\cargo-targets\\zircon-ai-m1`: 44 passed, 0 failed; warnings shown by the run are pre-existing shared `zircon_runtime` warnings, with no AI crate warnings.

The runtime manifest contribution test keeps static `plugin.toml`, linked descriptor metadata, and the built-in catalog in sync. The planned scoped verification commands are listed in the document header and should be rerun during the milestone testing stage.
